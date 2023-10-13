use crate::pages::generic_page::GenericPage;
use crate::u48::U48;
use std::fmt;

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
    pub fn get_total_allocated_pages(&self) -> U48 {
        let mut bytes: [u8; 6] = [0; 6]; // Initializing a byte array.
        bytes.copy_from_slice(&self.data[0..6]); // Copying the first 6 bytes from data.
        U48 { 0: bytes } // Returning a U48 instance with the copied bytes.
    }

    // Method to set the total allocated pages using a U48 instance.
    pub fn set_total_allocated_pages(&mut self, total_allocated_pages: U48) {
        self.data[0..6].copy_from_slice(&total_allocated_pages.0); // Copying the bytes from total_allocated_pages into the first 6 bytes of data.
    }

    pub fn get_free_pages_list(&self) -> U48 {
        let mut bytes: [u8; 6] = [0; 6]; // Initializing a byte array.
        bytes.copy_from_slice(&self.data[6..12]); // Copying the first 6 bytes from data.
        U48 { 0: bytes } // Returning a U48 instance with the copied bytes.
    }

    pub fn set_free_list_page_next(&mut self, free_pages_list: U48) {
        self.data[6..12].copy_from_slice(&free_pages_list.0); // Copying the bytes from total_allocated_pages into the first 6 bytes of data.
    }

    // Method to set the contents of this ConfigPage with the contents of another ConfigPage.
    pub fn set_config_page(&mut self, config: &ConfigPage) {
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
