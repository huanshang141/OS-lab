#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;
#[macro_use]
extern crate log;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    test();

    wait(spawn_init());
    ysos::shutdown();
}

pub fn spawn_init() -> proc::ProcessId {
    proc::spawn("shell").unwrap()
}
pub fn test() {
    use storage::PartitionTable;
    if let Some(drive) = ysos::drivers::ata::AtaDrive::open(0, 0) {
        match storage::mbr::MbrTable::parse(drive) {
            Ok(mbr_table) => {
                info!("MBR partition table parsed successfully");
                match mbr_table.partitions() {
                    Ok(partitions) => {
                        info!("Found {} active partitions", partitions.len());

                        for (i, partition) in partitions.iter().enumerate() {
                            info!("Partition {}: {:?}", i, partition);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get partitions: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse MBR table: {:?}", e);
            }
        }
    } else {
        error!("Failed to open ATA drive");
    }
}
