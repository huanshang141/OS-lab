#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use uefi::{Status, entry, fs::Path};
use x86_64::registers::control::*;
use ysos_boot::*;
mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    // 1. Load config
    let config = {
        // 1. 打开配置文件
        let mut file = fs::open_file(CONFIG_PATH);

        // 2. 加载文件内容到内存
        let buffer = fs::load_file(&mut file);

        // 3. 解析配置文件内容
        config::Config::parse(&buffer)
    };

    info!("Config: {:#x?}", config);

    // 2. Load ELF files
    let elf = {
        // 1. 从配置中获取内核路径
        let kernel_path = config.kernel_path;
        info!("Loading kernel from: {}", kernel_path);

        // 2. 打开内核文件
        let mut file = fs::open_file(kernel_path);

        // 3. 加载内核文件到内存
        let buffer = fs::load_file(&mut file);

        // 4. 解析ELF文件
        match xmas_elf::ElfFile::new(buffer) {
            Ok(elf_file) => {
                info!(
                    "Kernel ELF loaded, entry point: {:#x}",
                    elf_file.header.pt2.entry_point()
                );
                elf_file
            }
            Err(e) => {
                panic!("Failed to parse ELF file: {:?}", e);
            }
        }
    };

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is readonly, disable write protect (Cr0)

    // FIXME: map physical memory to specific virtual address offset

    // FIXME: load and map the kernel elf file

    // FIXME: map kernel stack

    // FIXME: recover write protect (Cr0)

    free_elf(elf);

    // 5. Pass system table to kernel
    let ptr = uefi::table::system_table_raw().expect("Failed to get system table");
    let system_table = ptr.cast::<core::ffi::c_void>();

    // 6. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let mmap = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    jump_to_entry(&bootinfo, stacktop);
}
