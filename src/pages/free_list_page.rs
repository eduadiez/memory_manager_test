use crate::memory_manager::MemoryManager;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::io::Cursor;
use std::{fmt, vec};

#[derive(PartialEq)]
pub struct FreeListPage<'a> {
    pub data: &'a mut [u8],
}

// Defining constants to avoid magic numbers
const FREE_LIST_PAGE_NEXT_START: usize = 0;
#[allow(dead_code)]
const FREE_LIST_PAGE_NEXT_END: usize = 6;
const DATA_START: usize = 16;
const DATA_END: usize = 4096;

impl<'a> FreeListPage<'a> {
    // Initialize free list pages
    #[allow(dead_code)]
    pub fn init_free_list_pages(
        &mut self,
        memory: &MemoryManager,
        num_pages: u64,
    ) -> Result<(), std::io::Error> {
        // Generate a vector of free page indices
        let mut free_page_indices: Vec<u64> = (1..num_pages).collect();

        // Calculate the number of chunks needed
        let num_chunks = (free_page_indices.len() + 679) / 680;
        let chunked_free_page_indices: Vec<_> = free_page_indices.drain(..num_chunks).collect();

        let mut free_list_pages: Vec<FreeListPage> = vec![];

        // Retrieve the free list pages from memory
        for (i, free_list_page) in chunked_free_page_indices.iter().enumerate() {
            free_list_pages.push(memory.get_page_mut::<FreeListPage>(*free_list_page)?);
            if i == num_chunks - 1 {
                // If we are at the last free list page, we need to set the free pages list to 0
                free_list_pages[i].set_free_list_page_next(0u64);
            } else {
                free_list_pages[i].set_free_list_page_next(chunked_free_page_indices[i + 1]);
            }
        }

        let bytes = &mut [0u8; 8]; // Un array de 8 bytes para almacenar u64

        // Process chunks of free pages
        for (i, chunk) in free_page_indices.chunks(680).enumerate() {
            let mut byte_vector: Vec<u8> = Vec::with_capacity(4080);
            for u48 in chunk {
                LittleEndian::write_u64(bytes, *u48); // Aquí escribes el u64 en el array de bytes
                byte_vector.extend_from_slice(bytes);
            }
            // Pad with zeros to meet the required length
            byte_vector.resize(4080, 0);
            // Set free pages list slice
            free_list_pages[i].set_free_list_page_data_slice(&byte_vector);
        }
        Ok(())
    }

    pub fn get_free_list_page_next(&self) -> u64 {
        let mut value = LittleEndian::read_u64(
            &self.data[FREE_LIST_PAGE_NEXT_START..FREE_LIST_PAGE_NEXT_START + 8],
        );
        // Suponiendo que quieras anular los últimos 2 bytes:
        let mask = !0u64 >> (8 * 2);
        value &= mask;

        value
    }

    pub fn set_free_list_page_next(&mut self, value: u64) {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, value);

        self.data[FREE_LIST_PAGE_NEXT_START..FREE_LIST_PAGE_NEXT_START + 6]
            .copy_from_slice(&buf[0..6]);
    }

    // Method to set the contents of this FreeListPage with the contents of another FreeListPage.
    pub fn get_free_pages_list_slice(&self) -> Result<[u64; 510], std::io::Error> {
        let mut u64_array = [0u64; 510];

        u64_array
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, u64_val)| {
                let offset = i * std::mem::size_of::<u64>();
                *u64_val = LittleEndian::read_u64(
                    &self.data
                        [DATA_START + offset..DATA_START + offset + std::mem::size_of::<u64>()],
                );
            });

        Ok(u64_array)
    }
    // Method to set the contents of this FreeListPage with the contents of another FreeListPage.
    #[allow(dead_code)]
    pub fn set_free_list_page_header_slice(&mut self, data_slice: &[u8]) {
        self.data[FREE_LIST_PAGE_NEXT_START..DATA_START].copy_from_slice(data_slice);
        // Copying the bytes from data_slice into the remaining bytes of data.
    }
    #[allow(dead_code)]
    pub fn copy_free_list_page_header_slice(&mut self, free_list_page: &FreeListPage) {
        self.data[FREE_LIST_PAGE_NEXT_START..DATA_START]
            .copy_from_slice(&free_list_page.data[FREE_LIST_PAGE_NEXT_START..DATA_START]);
        // Copying the bytes from data_slice into the remaining bytes of data.
    }
    // Method to set the contents of this FreeListPage with the contents of another FreeListPage.
    pub fn set_free_list_page_data_slice(&mut self, data_slice: &[u8]) {
        self.data[DATA_START..DATA_END].copy_from_slice(&data_slice);
        // Copying the bytes from data_slice into the remaining bytes of data.
    }
    pub fn get_recycled_pages_list(&self) -> Result<Vec<u64>, std::io::Error> {
        let mut cursor = Cursor::new(&self.data[DATA_START..DATA_END]);
        let mut vec = Vec::new();

        while let Ok(num) = cursor.read_u64::<LittleEndian>() {
            if num != 0 {
                vec.push(num);
            }
        }
        Ok(vec)
    }
}

// Implementing the Debug trait for FreeListPage to enable custom debug formatting.
impl<'a> fmt::Debug for FreeListPage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FreeListPage {{free_list_page_next: {:?} \nget_free_pages_list_slice: {:?} ...}}",
            self.get_free_list_page_next(),
            self.get_free_pages_list_slice().unwrap()[0..3].to_vec()
        )
    }
}
