use crate::memory_manager::MemoryManager;
use crate::pages::generic_page::GenericPage;
use crate::u48::U48;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::{fmt, vec};

#[derive(PartialEq)]
pub struct FreeListPage<'a> {
    pub data: &'a mut [u8],
}

// Defining constants to avoid magic numbers
const FREE_LIST_PAGE_NEXT_START: usize = 0;
const FREE_LIST_PAGE_NEXT_END: usize = 6;
const DATA_START: usize = 16;
const DATA_END: usize = 4096;

impl<'a> FreeListPage<'a> {
    // Static method to create a FreeListPage instance from a GenericPage instance.
    pub fn from_generic_page(page: GenericPage<'a>) -> Self {
        FreeListPage { data: page.data }
    }

    // Initialize free list pages
    pub fn init_free_list_pages(
        &mut self,
        memory: &MemoryManager,
        num_pages: u64,
    ) -> Result<(), std::io::Error> {
        // Generate a vector of free page indices
        let mut free_page_indices: Vec<U48> = (1..num_pages).map(U48::from).collect();

        // Calculate the number of chunks needed
        let num_chunks = (free_page_indices.len() + 679) / 680;
        let chunked_free_page_indices: Vec<_> = free_page_indices.drain(..num_chunks).collect();

        let mut free_list_pages: Vec<FreeListPage> = vec![];

        // Retrieve the free list pages from memory
        for (i, free_list_page) in chunked_free_page_indices.iter().enumerate() {
            free_list_pages.push(memory.get_page_mut::<FreeListPage>(free_list_page.to_usize())?);
            if i == num_chunks - 1 {
                // If we are at the last free list page, we need to set the free pages list to 0
                free_list_pages[i].set_free_list_page_next(U48::from(0u64));
            } else {
                free_list_pages[i].set_free_list_page_next(chunked_free_page_indices[i + 1]);
            }
        }

        // Process chunks of free pages
        for (i, chunk) in free_page_indices.chunks(680).enumerate() {
            let mut byte_vector: Vec<u8> = Vec::with_capacity(4080);
            for u48 in chunk {
                byte_vector.extend_from_slice(u48.to_bytes());
            }
            // Pad with zeros to meet the required length
            byte_vector.resize(4080, 0);
            // Set free pages list slice
            free_list_pages[i].set_free_list_page_data_slice(&byte_vector);
        }
        Ok(())
    }
    pub fn get_free_pages_list_ptr(
        init_index: U48,
        memory: &MemoryManager,
    ) -> Result<Vec<U48>, std::io::Error> {
        let mut free_pages_list = vec![];
        let mut current_index = init_index;
        loop {
            free_pages_list.push(current_index);
            let next_free_list_page =
                memory.get_page_mut::<FreeListPage>(current_index.to_usize())?;
            current_index = next_free_list_page.get_free_list_page_next();
            if current_index == U48::from(0u64) {
                break;
            }
        }
        Ok(free_pages_list)
    }

    pub fn get_free_pages_list_vec(
        init_index: U48,
        memory: &MemoryManager,
    ) -> Result<Vec<U48>, std::io::Error> {
        let mut free_pages_list = vec![];
        let mut free_pages_list_vec = vec![];

        let mut current_index = init_index;
        loop {
            free_pages_list.push(current_index);
            let mut next_free_list_page =
                memory.get_page_mut::<FreeListPage>(current_index.to_usize())?;

            let free_pages_list_slice = next_free_list_page.get_free_pages_list_slice()?;
            free_pages_list_vec.extend(free_pages_list_slice.to_vec());
            current_index = next_free_list_page.get_free_list_page_next();
            if current_index == U48::from(0u64) {
                break;
            }
            if free_pages_list_vec.len() > 100000 {
                break;
            }
        }
        Ok(free_pages_list_vec)
    }

    pub fn get_free_list_page_next(&self) -> U48 {
        U48::copy_slice_to_u48(&self.data[FREE_LIST_PAGE_NEXT_START..FREE_LIST_PAGE_NEXT_END])
    }

    pub fn set_free_list_page_next(&mut self, next_free_list_page_index: U48) {
        self.data[FREE_LIST_PAGE_NEXT_START..FREE_LIST_PAGE_NEXT_END]
            .copy_from_slice(&next_free_list_page_index.0); // Copying the bytes from next_free_list_page_index into the first 6 bytes of data.
    }

    // Method to set the contents of this FreeListPage with the contents of another FreeListPage.
    pub fn get_free_pages_list_slice(&self) -> Result<[U48; 680], std::io::Error> {
        let mut u48_array = [U48::default(); 680];
        u48_array.par_iter_mut().enumerate().for_each(|(i, u48)| {
            let start = i * std::mem::size_of::<U48>();
            let end = start + std::mem::size_of::<U48>();
            *u48 = U48::from_bytes_range(&self.data[DATA_START..DATA_END], start, end);
        });
        Ok(u48_array)
    }
    // Method to set the contents of this FreeListPage with the contents of another FreeListPage.
    pub fn set_free_list_page_data_slice(&mut self, data_slice: &[u8]) {
        self.data[DATA_START..DATA_END].copy_from_slice(data_slice); // Copying the bytes from data_slice into the remaining bytes of data.
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
