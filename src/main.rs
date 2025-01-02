use std::alloc::Layout;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SDError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Invalid block size")]
    InvalidBlockSize,
    #[error("Read error: expected {expected} bytes got {actual}")]
    ReadError { expected: usize, actual: usize},
}

#[derive(Debug)]
pub struct FATBootSector {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    number_of_fats: u8,
    root_dir_entries: u16,
    total_sectors_16: u16,
    media_descriptor: u8,
    sectors_per_fat: u16,
    total_sectors_32: u32,
}

#[derive(Debug)]
pub struct FATLayout {
    fat_start: u32,
    root_dir_start: u32,
    data_start: u32,
}

pub struct SDController{
    device: File,
    block_size: usize,
}

impl SDController{
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, SDError>{
        let device = File::open(path)?;
        Ok(SDController { device, block_size: 512})
    }
    pub fn read_block(&mut self, block_index: u32) -> Result<Vec<u8>, SDError> {
        let mut buffer = vec![0; self.block_size];

        let position = block_index as u64 * self.block_size as u64;

        self.device.seek(SeekFrom::Start(position))?;

        let bytes_read = self.device.read(&mut buffer)?;
        if bytes_read != self.block_size {
            return Err(SDError::ReadError {
                expected: self.block_size,
                actual: bytes_read,
            });
        }
        Ok(buffer)

    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn read_boot_sector(&mut self) -> Result<FATBootSector, SDError> {
        let data = self.read_block(0)?;

        Ok(FATBootSector{
            bytes_per_sector: u16::from_le_bytes([data[11], data[12]]),
            sectors_per_cluster: data[13],
            reserved_sectors: u16::from_le_bytes([data[14], data[15]]),
            number_of_fats: data[16],
            root_dir_entries: u16::from_le_bytes([data[17], data[18]]),
            total_sectors_16: u16::from_le_bytes([data[19], data[20]]),
            media_descriptor: data[21],
            sectors_per_fat: u16::from_le_bytes([data[22], data[23]]),
            total_sectors_32: u32::from_le_bytes([data[32], data[33], data[34], data[36]]),
        })

    }

    pub fn calculate_layout(&self, boot_sector: &FATBootSector) -> FATLayout{
        let root_dir_sectors = ((boot_sector.root_dir_entries as u32 * 32) +
                                (boot_sector.bytes_per_sector as u32 -1)) /
                                boot_sector.bytes_per_sector as u32;
        let fat_start = boot_sector.reserved_sectors as u32;
        let root_dir_start = fat_start +
                                (boot_sector.number_of_fats as u32 * boot_sector.sectors_per_fat as u32);
        let data_start = root_dir_start + root_dir_sectors;

        FATLayout {
            fat_start,
            root_dir_start,
            data_start,
        }

    }


}

fn main() -> Result<(), SDError>{

    println!("Device path selected /dev/disk4");
    let mut controller = SDController::new("/dev/rdisk4s1")?;
    println!("Succesfully opened SD Card");

    match controller.read_block(0){
        Ok(data) => {
            println!("Succesfully read first block:");
            for (i, byte) in data.iter().take(16).enumerate(){
                if i % 16 == 0 {
                    print!("\n{:04x}: ", i);
                }
                print!("{:02x} ", byte);
            }
        }
        Err(e) => println!("Failed to read block: {}", e),
    }

    match controller.read_boot_sector() {
        Ok(boot_sector) => {
            println!("\nFAT16 Boot Sector Information:");
            println!("Bytes per sector: {}", boot_sector.bytes_per_sector);
            println!("Sectors per cluster {}", boot_sector.sectors_per_cluster);
            println!("Reserved sectors {}", boot_sector.reserved_sectors);
            println!("Number of FATs: {}", boot_sector.number_of_fats);
            println!("Root directory entries: {}", boot_sector.root_dir_entries);
            println!("Total sectors: {}",
                if boot_sector.total_sectors_16 > 0{
                    boot_sector.total_sectors_16 as u32
                } else {
                    boot_sector.total_sectors_32
                });
            println!("Sectors per FAT: {}", boot_sector.sectors_per_fat);

            let layout = controller.calculate_layout(&boot_sector);
            println!("\nFilesystem layout:");
            println!("FAT starts at sector: {}", layout.fat_start);
            println!("Root directory starts at sector: {}", layout.root_dir_start);
            println!("Data area starts at sector: {}", layout.data_start);

        }
        Err(e) => println!("Failed to read boot sector: {}", e),
    }

    Ok(())
}
