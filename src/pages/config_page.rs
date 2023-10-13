use crate::pages::generic_page::GenericPage;
use crate::u48::U48;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::fmt;

// Defining constants to avoid magic numbers
const TOTAL_ALLOCATED_PAGES_START: usize = 0;
const TOTAL_ALLOCATED_PAGES_END: usize = 6;
const LAST_USED_PAGE_START: usize = 6;
const LAST_USED_PAGE_END: usize = 14;
const RECYCLED_PAGES_LIST_START: usize = 12;
const RECYCLED_PAGES_LIST_END: usize = 18;
const DATA_START: usize = 16;
const DATA_END: usize = 4096;

#[derive(PartialEq)]
pub struct ConfigPage<'a> {
    pub data: &'a mut [u8],
}

impl<'a> ConfigPage<'a> {
    // Static method to create a ConfigPage instance from a GenericPage instance.
    pub fn from_generic_page(page: GenericPage<'a>) -> Self {
        ConfigPage { data: page.data }
    }

    // Method to retrieve the total allocated pages from the first 6 bytes of data.
    pub fn get_total_allocated_pages(&self) -> u64 {
        let mut value = LittleEndian::read_u64(
            &self.data[TOTAL_ALLOCATED_PAGES_START..TOTAL_ALLOCATED_PAGES_START + 8],
        );
        // Suponiendo que quieras anular los últimos 2 bytes:
        let mask = !0u64 >> (8 * 2);
        value &= mask;

        value
    }

    // Method to set the total allocated pages using a U48 instance.
    pub fn set_total_allocated_pages(&mut self, value: u64) {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, value);

        self.data[TOTAL_ALLOCATED_PAGES_START..TOTAL_ALLOCATED_PAGES_START + 6]
            .copy_from_slice(&buf[0..6]);
    }

    pub fn get_last_used_page(&self) -> u64 {
        let mut value =
            LittleEndian::read_u64(&self.data[LAST_USED_PAGE_START..LAST_USED_PAGE_START + 8]);
        // Suponiendo que quieras anular los últimos 2 bytes:
        let mask = !0u64 >> (8 * 2);
        value &= mask;

        value
    }
    pub fn set_last_used_page(&mut self, value: u64) {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, value);

        self.data[LAST_USED_PAGE_START..LAST_USED_PAGE_START + 6].copy_from_slice(&buf[0..6]);
    }

    pub fn get_recycled_pages_list(&self) -> u64 {
        let mut value = LittleEndian::read_u64(
            &self.data[RECYCLED_PAGES_LIST_START..RECYCLED_PAGES_LIST_START + 8],
        );
        // Suponiendo que quieras anular los últimos 2 bytes:
        let mask = !0u64 >> (8 * 2);
        value &= mask;

        value
    }

    pub fn set_recycled_pages_list(&mut self, free_pages_list: u64) {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, free_pages_list);
        self.data[RECYCLED_PAGES_LIST_START..RECYCLED_PAGES_LIST_END].copy_from_slice(&buf[0..6]);
    }

    // Method to set the contents of this ConfigPage with the contents of another ConfigPage.
    pub fn copy_config_page(&mut self, config: &ConfigPage) {
        self.data[0..4096].copy_from_slice(&config.data[0..4096]); // Copying the data from config into self.
    }
}

// Implementing the Debug trait for ConfigPage to enable custom debug formatting.
impl<'a> fmt::Debug for ConfigPage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Custom formatting to display the result of get_total_allocated_pages when formatting ConfigPage for debugging.
        write!(
            f,
            "ConfigPage {{ get_total_allocated_pages: {:?} }}",
            &self.get_total_allocated_pages()
        )
    }
}
