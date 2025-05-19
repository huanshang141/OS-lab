use super::*;
use crate::{
    memory::{
        self, PAGE_SIZE,
        allocator::{ALLOCATOR, HEAP_SIZE},
        get_frame_alloc_for_sure,
    },
    proc::vm::stack::STACK_INIT_TOP,
};
use alloc::{
    collections::*,
    format,
    sync::{Arc, Weak},
};
use spin::{Mutex, RwLock};
use x86::current;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {
    init.write().resume();
    processor::set_pid(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
    wait_queue: Mutex<BTreeMap<ProcessId, BTreeSet<ProcessId>>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: app_list,
            wait_queue: Mutex::new(BTreeMap::new()),
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        let proc = self.current();
        let mut proc_write = proc.write();
        proc_write.tick();
        proc_write.save(context);
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
        let next_pid = loop {
            if let Some(pid) = self.ready_queue.lock().pop_front() {
                if let Some(proc) = self.get_proc(&pid) {
                    if proc.read().status() == ProgramStatus::Ready {
                        break pid;
                    }
                }
            } else {
                break processor::get_pid();
            }
        };

        if let Some(proc) = self.get_proc(&next_pid) {
            let mut proc_write = proc.write();
            proc_write.resume();
            proc_write.restore(context);
        }

        processor::set_pid(next_pid);

        next_pid
    }

    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc_vm = Some(ProcessVm::new(page_table));
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);

    //     // 获取新进程的PID
    //     let pid = proc.pid();

    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();
    //     proc.write().pause();
    //     // 设置栈帧

    //     proc.write().set_stack_frame(entry, stack_top);

    //     // 添加到进程映射表
    //     self.add_proc(pid, proc.clone());

    //     // 将进程添加到就绪队列
    //     self.push_ready(pid);

    //     // 返回新进程PID
    //     pid
    // }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            let current = self.current();
            let mut inner = current.write();

            trace!(
                "Handling page fault at {:#x} for process {}",
                addr.as_u64(),
                current.pid()
            );

            inner.handle_page_fault(addr)
        } else {
            warn!(
                "Illegal page fault: {:?} at {:#x} for process #{}",
                err_code,
                addr,
                self.current().pid()
            );
            false
        }
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);

        if let Some(pids) = self.wait_queue.lock().remove(&pid) {
            for waiter_pid in pids {
                self.wake_up(waiter_pid, Some(ret));
                trace!(
                    "Woken up process #{} that was waiting for #{}",
                    waiter_pid, pid
                );
            }
        }
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{}\n", p).as_str());

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        //avoid deadlock
        x86_64::instructions::interrupts::without_interrupts(|| {
            self.get_proc(&pid).and_then(|proc| proc.read().exit_code())
        })
    }
    pub fn app_list(&self) -> boot::AppListRef {
        self.app_list
    }
    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let page_table_mapper: x86_64::structures::paging::OffsetPageTable<'static> =
            page_table.mapper();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, parent, proc_vm, proc_data);

        let pid = proc.pid();

        let mut inner = proc.write();
        // 加载 ELF 文件
        inner.load_elf(elf, page_table_mapper, pid);
        debug!("Load ELF");
        // inner.set_stack_frame(
        //     VirtAddr::new_truncate(elf.header.pt2.entry_point()),
        //     VirtAddr::new_truncate(STACK_INIT_TOP),
        // );

        // 将进程标记为就绪状态
        inner.pause();
        drop(inner);

        trace!("New {:#?}", &proc);

        // 添加到进程映射表
        self.add_proc(pid, proc.clone());

        // 将进程添加到就绪队列
        self.push_ready(pid);

        pid
    }
    pub fn fork(&self) {
        let current = self.current();
        let child = current.fork();
        self.push_ready(child.pid());
        self.add_proc(child.pid(), child);

        debug!("Ready queue: {:?}", self.ready_queue.lock());
    }
    pub fn block(&self, pid: ProcessId) {
        if let Some(proc) = self.get_proc(&pid) {
            let mut proc_write = proc.write();
            proc_write.block();
            trace!("Process #{} blocked", pid);
        }
    }
    pub fn wait_pid(&self, pid: ProcessId) {
        let current_pid = processor::get_pid();

        if self.get_proc(&pid).is_none() {
            debug!("Process #{} not found, cannot wait", pid);
            return;
        }

        let mut wait_queue = self.wait_queue.lock();
        wait_queue
            .entry(pid)
            .or_insert(BTreeSet::new())
            .insert(current_pid);

        trace!("Process #{} is waiting for process #{}", current_pid, pid);
    }
    /// Wake up the process with the given pid
    ///
    /// If `ret` is `Some`, set the return value of the process
    pub fn wake_up(&self, pid: ProcessId, ret: Option<isize>) {
        if let Some(proc) = self.get_proc(&pid) {
            let mut inner = proc.write();
            if let Some(ret) = ret {
                inner.set_rax(ret as usize);
            }
            // 将进程状态设置为就绪
            inner.pause();
            drop(inner);

            // 将进程添加到就绪队列
            self.push_ready(pid);

            trace!("Process #{} woken up", pid);
        }
    }
}
