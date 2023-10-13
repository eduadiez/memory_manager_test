mod logger;
mod memory_manager;
mod pages;
mod u48;

extern crate slog;
extern crate slog_term;

use crate::memory_manager::MemoryManager;
use crate::pages::any_page::AnyPage;
use crate::pages::config_page::ConfigPage;
use crate::pages::free_list_page::FreeListPage;
use crate::pages::generic_page::GenericPage;
use crate::pages::page_manager::PageManager;
use crate::u48::U48;
use slog::info;
use std::mem;
use std::time::Instant;

fn main() -> Result<(), std::io::Error> {
    let log = logger::get_logger();
    let num_pages = 1000u64;
    let mut memory: MemoryManager = MemoryManager::new("test.bin", num_pages)?;

    let mut page_manager = PageManager::new(&mut memory, num_pages)?;

    page_manager.get_free_pages(15);
    let mut pending_recycled = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10].to_vec();
    page_manager.recyle_pages(&mut pending_recycled);
    page_manager.consolidate_state()?;

    return Ok(());
    /* let mut data: [u8; 4096] = [0; 4096];
    let mut config_page_editable: ConfigPage<'_> = ConfigPage { data: &mut data };
    config_page_editable.set_total_allocated_pages(1234u64);

    let generic_page: GenericPage<'_> = memory.get_page_mut(0)?;
    let any_page: AnyPage<'_> = AnyPage::Generic(generic_page);

    if let Some(mut config_page) = any_page.to_config_page() {
        println!("Converted to ConfigPage: {:?}", config_page);
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );
        config_page.set_total_allocated_pages(2000);
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );

        config_page.copy_config_page(&config_page_editable);
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );
    } else {
        println!("Failed to convert to ConfigPage");
    }
    memory.flush()?;
    Ok(()) */
}
