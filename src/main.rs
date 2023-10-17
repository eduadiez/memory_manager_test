mod logger;
mod memory_manager;
mod pages;

extern crate slog;
extern crate slog_term;

use crate::memory_manager::MemoryManager;
use crate::pages::page_manager::PageManager;
use slog::debug;

fn main() -> Result<(), std::io::Error> {
    let log = logger::get_logger();
    let num_pages = 1000u64;

    let mut memory: MemoryManager = MemoryManager::new("test.bin", num_pages)?;

    let mut page_manager: PageManager<'_> = PageManager::new(&mut memory, num_pages)?;

    let mut pending_recycled = page_manager.get_free_pages(30, true)?;
    page_manager.recyle_pages(&mut pending_recycled);
    page_manager.consolidate_state()?;
    
    for i in 0..page_manager.config_page.get_version_number() {
        let _ptr = page_manager.config_page.get_recycled_pages_list_at(i);
        //debug!(log, "get_recycled_pages_list_at({}) -> {:?}", i, ptr);
        let vec = page_manager.get_free_list_page_at(i)?;
        let fresh_pages = page_manager.config_page.get_last_used_page_at(i);
        debug!(log, "last_used_page {} get_free_pages({}) -> {:?}",fresh_pages,  i, vec)
    }

    //

    debug!(log, "{:?} ", page_manager.config_page);
    return Ok(());
}
