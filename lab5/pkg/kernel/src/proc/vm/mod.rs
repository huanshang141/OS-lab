use alloc::format;
use x86_64::{
    VirtAddr,
    structures::paging::{mapper::MapToError, page::*, *},
};

use crate::{humanized_size, memory::*};
use xmas_elf::ElfFile;

pub mod stack;

use self::stack::*;

use super::{PageTableContext, ProcessId};

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
        // 计算基于PID的栈顶地址
        // 栈顶位置 = STACK_MAX - (pid-1) * STACK_MAX_SIZE - 8
        use self::stack::{STACK_DEF_PAGE, STACK_MAX, STACK_MAX_SIZE};

        let stack_top = STACK_INIT_TOP - ((pid.0 as u64 - 1) * STACK_MAX_SIZE);
        let stack_bot = STACK_INIT_BOT - ((pid.0 as u64 - 1) * STACK_MAX_SIZE);
        // let stack_bot = stack_top - STACK_DEF_PAGE * crate::memory::PAGE_SIZE + 1;

        let stack_top_addr = VirtAddr::new(stack_top);
        // 获取页表映射器和帧分配器
        let mapper = &mut self.page_table.mapper();
        let frame_allocator = &mut *get_frame_alloc_for_sure();

        // 使用elf::map_range分配和映射栈空间
        let page_range = match elf::map_range(
            stack_bot,
            STACK_DEF_PAGE,
            mapper,
            frame_allocator,
            true,
            false,
        ) {
            Ok(range) => range,
            Err(e) => {
                error!("Failed to map stack: {:?}", e);
                panic!("Failed to allocate stack for process {}", pid.0);
            }
        };

        // 更新进程的栈信息
        self.stack = Stack::new(
            Page::containing_address(VirtAddr::new(stack_top)),
            STACK_DEF_PAGE,
        );

        trace!(
            "Process stack allocated at {:#x}-{:#x}",
            stack_bot, stack_top
        );

        stack_top_addr
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub fn load_elf(
        &mut self,
        elf: &ElfFile,
        mut mapper: x86_64::structures::paging::OffsetPageTable<'static>,
        pid: ProcessId,
    ) -> Result<VirtAddr, MapToError<Size4KiB>> {
        // 初始化进程栈并获取栈顶地址
        let stack_top = self.init_proc_stack(pid);
        // 获取页表映射器和帧分配器
        let frame_allocator = &mut *get_frame_alloc_for_sure();

        // 加载ELF文件到内存，设置为用户可访问
        elf::load_elf(
            elf,
            *PHYSICAL_OFFSET.get().unwrap(),
            &mut mapper,
            frame_allocator,
            true, // 设置USER_ACCESSIBLE标志
        )?;

        // 返回栈顶地址
        Ok(stack_top)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }
    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
        }
    }
    pub fn stack_bot(&self) -> VirtAddr {
        self.stack.stack_bot()
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
