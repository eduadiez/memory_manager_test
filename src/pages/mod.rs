pub mod generic_page;
pub mod config_page;
pub mod free_list_page;
pub mod from_slice;
pub mod page_manager;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Page {
    pub data: [u8; 4096],
}



