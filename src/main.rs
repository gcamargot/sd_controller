use std::fs::File;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SDError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Device not found")]
    DeviceNotFound,
}

pub struct SDController{
    device: File,
}

impl SDController{
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, SDError>{
        let device = File::open(path)?;
        Ok(SDController { device })
    }

}

fn main() -> Result<(), SDError>{
    let controller = SDController::new("/dev/disk4");
    println!("Succesfully opened SD Card");
    Ok(())
}
