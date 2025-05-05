use super::*;
use crate::memory::{
    self, PAGE_SIZE,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure,
};
use alloc::{
    collections::*,
    format,
    sync::{Arc, Weak},
};
use spin::{Mutex, RwLock};

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

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);

        // 获取新进程的PID
        let pid = proc.pid();

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();
        proc.write().pause();
        // 设置栈帧

        proc.write().set_stack_frame(entry, stack_top);

        // 添加到进程映射表
        self.add_proc(pid, proc.clone());

        // 将进程添加到就绪队列
        self.push_ready(pid);

        // 返回新进程PID
        pid
    }

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
        inner.load_elf(elf, page_table_mapper);

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
}
