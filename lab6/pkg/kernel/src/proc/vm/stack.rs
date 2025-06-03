use elf;
use x86_64::{
    VirtAddr,
    structures::paging::{Mapper, Page, Translate, mapper::MapToError, page::*},
};

use super::{FrameAllocatorRef, MapperRef};

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x4000_0000_0000;
pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * crate::memory::PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_BOT: u64 = STACK_MAX - STACK_MAX_SIZE;
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;

const STACK_INIT_TOP_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(STACK_INIT_TOP));

// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_BOT: u64 = KSTACK_MAX - STACK_MAX_SIZE;
pub const KSTACK_DEF_PAGE: u64 = 512;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

const KSTACK_INIT_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(KSTACK_INIT_BOT));
const KSTACK_INIT_TOP_PAGE: Page<Size4KiB> =
    Page::containing_address(VirtAddr::new(KSTACK_INIT_TOP));

pub struct Stack {
    range: PageRange<Size4KiB>,
    usage: u64,
}

impl Stack {
    pub fn new(top: Page, size: u64) -> Self {
        Self {
            range: Page::range(top - size + 1, top + 1),
            usage: size,
        }
    }

    pub const fn empty() -> Self {
        Self {
            range: Page::range(STACK_INIT_TOP_PAGE, STACK_INIT_TOP_PAGE),
            usage: 0,
        }
    }

    pub const fn kstack() -> Self {
        Self {
            range: Page::range(KSTACK_INIT_PAGE, KSTACK_INIT_TOP_PAGE),
            usage: KSTACK_DEF_PAGE,
        }
    }

    pub fn init(&mut self, mapper: MapperRef, alloc: FrameAllocatorRef) {
        debug_assert!(self.usage == 0, "Stack is not empty.");

        self.range =
            elf::map_range(STACK_INIT_BOT, STACK_DEF_PAGE, mapper, alloc, true, false).unwrap();
        self.usage = STACK_DEF_PAGE;
    }

    pub fn handle_page_fault(
        &mut self,
        addr: VirtAddr,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> bool {
        if !self.is_on_stack(addr) {
            return false;
        }

        if let Err(m) = self.grow_stack(addr, mapper, alloc) {
            error!("Grow stack failed: {:?}", m);
            return false;
        }

        true
    }

    fn is_on_stack(&self, addr: VirtAddr) -> bool {
        let addr = addr.as_u64();
        let cur_stack_bot = self.range.start.start_address().as_u64();
        trace!("Current stack bot: {:#x}", cur_stack_bot);
        trace!("Address to access: {:#x}", addr);
        addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
    }

    fn grow_stack(
        &mut self,
        addr: VirtAddr,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Result<(), MapToError<Size4KiB>> {
        debug_assert!(self.is_on_stack(addr), "Address is not on stack.");

        let page = Page::containing_address(addr);
        if let x86_64::structures::paging::mapper::TranslateResult::Mapped { .. } =
            mapper.translate(page.start_address())
        {
            return Ok(());
        }

        let current_base = self.range.start.start_address().as_u64();
        let new_base = page.start_address().as_u64();
        let stack_size = self.range.end.start_address().as_u64() - new_base;

        if stack_size > STACK_MAX_SIZE {
            error!("Stack overflow: attempted to grow beyond maximum size");
            return Err(MapToError::FrameAllocationFailed);
        }

        let page_addr = page.start_address().as_u64();
        let _ = elf::map_range(page_addr, 1, mapper, alloc, true, false)?;

        if new_base < current_base {
            let pages_added = (current_base - new_base) / page.size();
            self.range = Page::range(page, self.range.end);
            self.usage += pages_added as u64;

            trace!(
                "Stack grown by {} pages, now at {:#x}-{:#x}",
                pages_added,
                self.range.start.start_address().as_u64(),
                self.range.end.start_address().as_u64()
            );
        }

        Ok(())
    }

    pub fn memory_usage(&self) -> u64 {
        self.usage * crate::memory::PAGE_SIZE
    }

    pub fn stack_top(&self) -> u64 {
        self.range.end.start_address().as_u64()
    }
    pub fn stack_bot(&self) -> u64 {
        self.range.start.start_address().as_u64()
    }

    pub fn fork(
        &self,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
        stack_offset_count: u64,
    ) -> Self {
        let stack_offset = stack_offset_count * STACK_MAX_SIZE;
        let parent_stack_bot = self.range.start.start_address().as_u64();
        let parent_stack_top = self.range.end.start_address().as_u64();
        let mut child_stack_bot = parent_stack_bot - stack_offset;
        let child_stack_range;
        let child_stack_size = self.usage;

        loop {
            match elf::map_range(
                child_stack_bot,
                child_stack_size,
                mapper,
                alloc,
                true,
                false,
            ) {
                Ok(range) => {
                    child_stack_range = range;
                    break;
                }
                Err(_) => {
                    trace!("Map thread stack to {:#x} failed.", child_stack_bot);
                    child_stack_bot -= STACK_MAX_SIZE; // stack grow down
                }
            }
        }
        let parent_stack_addr = parent_stack_bot;
        let child_stack_addr = child_stack_bot;
        self.clone_range(parent_stack_addr, child_stack_addr, child_stack_size);

        debug!(
            "Forked stack: child({:#x}-{:#x}), size: {} pages",
            child_stack_bot,
            child_stack_range.end.start_address().as_u64(),
            child_stack_size
        );

        Self {
            range: child_stack_range,
            usage: child_stack_size,
        }
    }

    /// Clone a range of memory
    ///
    /// - `src_addr`: the address of the source memory
    /// - `dest_addr`: the address of the target memory
    /// - `size`: the count of pages to be cloned
    fn clone_range(&self, cur_addr: u64, dest_addr: u64, size: u64) {
        trace!("Clone range: {:#x} -> {:#x}", cur_addr, dest_addr);
        unsafe {
            core::ptr::copy_nonoverlapping(
                cur_addr as *const u8,
                dest_addr as *mut u8,
                (size * Size4KiB::SIZE) as usize,
            );
        }
    }
}

impl core::fmt::Debug for Stack {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Stack")
            .field(
                "top",
                &format_args!("{:#x}", self.range.end.start_address().as_u64()),
            )
            .field(
                "bot",
                &format_args!("{:#x}", self.range.start.start_address().as_u64()),
            )
            .finish()
    }
}
