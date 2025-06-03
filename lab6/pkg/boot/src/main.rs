#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use uefi::mem::memory_map::MemoryMap;
use uefi::{Status, entry};
use x86_64::registers::control::*;
use ysos_boot::*;
mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Debug);
    info!("Running UEFI bootloader...");

    // 1. Load config
    let mut file = fs::open_file(CONFIG_PATH);
    let buffer = fs::load_file(&mut file);
    let config = config::Config::parse(&buffer);
    info!("Config: {:#x?}", config);

    // 2. Load ELF files

    let kernel_path = config.kernel_path;

    info!("Loading kernel from: {}", kernel_path);
    let mut file = fs::open_file(kernel_path);

    let buffer = fs::load_file(&mut file);
    let elf = match xmas_elf::ElfFile::new(buffer) {
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
    };

    set_entry(elf.header.pt2.entry_point() as usize);

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
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
        info!("Write protect disabled");
    }

    // FIXME: map physical memory to specific virtual address offset
    // let FrameAllocator = UEFIFrameAllocator();

    elf::map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut UEFIFrameAllocator {},
    );
    info!(
        "Physical memory mapped to offset {:#x}",
        config.physical_memory_offset
    );

    elf::load_elf(
        &elf,
        config.physical_memory_offset,
        &mut page_table,
        &mut UEFIFrameAllocator {},
        false, // kernel mode
    )
    .expect("Failed to load kernel ELF");
    info!("Kernel ELF loaded and mapped");

    // FIXME: map kernel stack
    // llm assist
    elf::map_range(
        config.kernel_stack_address, // 从配置文件获取栈地址
        config.kernel_stack_size,    // 从配置文件获取栈大小（页数）
        &mut page_table,             // 页表
        &mut UEFIFrameAllocator {},  // 帧分配器
        false,
        false,
    )
    .expect("Failed to map kernel stack");
    info!(
        "Kernel stack mapped at {:#x} with size {} pages",
        config.kernel_stack_address, config.kernel_stack_size
    );

    // FIXME: recover write protect (Cr0)
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
        info!("Write protect restored");
    }

    free_elf(elf);

    // 5. Pass system table to kernel
    let ptr = uefi::table::system_table_raw().expect("Failed to get system table");
    let system_table = ptr.cast::<core::ffi::c_void>();
    //load apps
    let apps = if config.load_apps {
        info!("Loading apps...");
        Some(load_apps())
    } else {
        info!("Skip loading apps");
        None
    };
    // 6. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let mmap = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table,
        loaded_apps: apps,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    jump_to_entry(&bootinfo, stacktop);
}
