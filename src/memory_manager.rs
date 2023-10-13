use crate::logger;
use memmap2::{MmapOptions, MmapRaw};
use slog::{crit, info};
use std::fs::OpenOptions;
use std::io::{self, ErrorKind};
use std::process;
use std::slice;

use crate::pages::from_slice::FromSlice;

const PAGE_SIZE: usize = 0x1000; // 4KB
const _FRAGMENT_SIZE: usize = 0x10; // 4KB
pub const RESERVED_CONFIG_PAGE_INDEX: u64 = 0;

#[derive(Debug)]
pub struct MemoryManager {
    mmap: MmapRaw,
}

impl MemoryManager {
    pub fn new(filename: &str, num_pages: u64) -> Result<Self, std::io::Error> {
        let log = logger::get_logger();
        // Open the memory-mapped file
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
        {
            Ok(file) => {
                info!(log, "File {} opened", filename);
                file
            }
            Err(e) => {
                crit!(log, "Failed to open file: {} - {}", filename, e);
                process::exit(1);
            }
        };
        // Set the file size
        let file_size: u64 = PAGE_SIZE as u64 * num_pages;
        file.set_len(file_size).map_err(|e| {
            let err_msg = format!("Failed to set file size: {} - {}", file_size, e);
            crit!(log, "{}", &err_msg);
            io::Error::new(ErrorKind::Other, err_msg)
        })?;
        info!(log, "File size: {:?} MB", file_size as f64 / 1_048_576.0);

        // Open a memory map for the file
        let mmap = MmapOptions::new().map_raw(&file).map_err(|e| {
            let err_msg = format!("Failed to create memory map: {}", e);
            crit!(log, "{}", &err_msg);
            io::Error::new(ErrorKind::Other, err_msg)
        })?;

        info!(log, "Correctly mapped {} pages into memory", num_pages);

        Ok(MemoryManager { mmap: mmap })
    }

    pub fn get_page_mut<'a, T: FromSlice<'a>>(&self, index: usize) -> Result<T, std::io::Error> {
        let offset = (index * PAGE_SIZE).try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Index {} is out of bounds", index),
            )
        })?;
        unsafe {
            let data = slice::from_raw_parts_mut(self.mmap.as_mut_ptr().offset(offset), PAGE_SIZE);
            Ok(T::from_slice(data))
        }
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.mmap.flush().map_err(|e| {
            let err_msg = format!("Flush has failed: {}", e);
            io::Error::new(ErrorKind::Other, err_msg)
        })?;
        Ok(())
    }
}
