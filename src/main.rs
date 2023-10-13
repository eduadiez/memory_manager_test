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
use crate::u48::U48;
use slog::{debug, info};
use std::time::Instant;

fn main() -> Result<(), std::io::Error> {
    let log = logger::get_logger();
    let num_pages = 10000000u64;
    let memory = MemoryManager::new("test.bin", num_pages)?;

    //Check if we need to init the memory
    let mut init_config_page =
        memory.get_page_mut::<ConfigPage>(memory_manager::RESERVED_CONFIG_PAGE_INDEX as usize)?;

    if init_config_page.get_total_allocated_pages() == U48::from(0u64) {
        info!(log, "Initializing memory...");
        init_config_page.set_total_allocated_pages(U48::from(num_pages));

        // We need to initialize the free pages list
        let mut free_list_page = memory.get_page_mut::<FreeListPage>(1)?;
        init_config_page.set_free_list_page_next(U48::from(1u64));

        let mut start_time = Instant::now();
        free_list_page.init_free_list_pages(&memory, num_pages)?;
        let mut duration = start_time.elapsed();

        info!(
            log,
            "Execution time of init_free_list_pages(): {:?}", duration
        );

        memory.flush()?;
        start_time = Instant::now();
        let a = FreeListPage::get_free_pages_list_ptr(U48::from(1u64), &memory)?;
        duration = start_time.elapsed();
        info!(
            log,
            "Execution time of get_free_pages_list_ptr(): {:?}", duration
        );

        println!("Free list: {:?}", a[0]);
        start_time = Instant::now();
        let b = FreeListPage::get_free_pages_list_vec(U48::from(1u64), &memory)?;
        duration = start_time.elapsed();
        info!(
            log,
            "Execution time of get_free_pages_list_ptr(): {:?}", duration
        );

        println!("Free list: {:?}", b[0])
    }
    info!(
        log,
        "Memory already initialized, total allocated pages: {:?}",
        init_config_page.get_total_allocated_pages().to_usize()
    );

    return Ok(());
    let mut data: [u8; 4096] = [0; 4096];
    let mut config_page_editable: ConfigPage<'_> = ConfigPage { data: &mut data };
    config_page_editable.set_total_allocated_pages(U48::from(1234u64));

    let generic_page: GenericPage<'_> = memory.get_page_mut(0)?;
    let any_page: AnyPage<'_> = AnyPage::Generic(generic_page);

    if let Some(mut config_page) = any_page.to_config_page() {
        println!("Converted to ConfigPage: {:?}", config_page);
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );
        config_page.set_total_allocated_pages(U48::from(2000u64));
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );

        config_page.set_config_page(&config_page_editable);
        println!(
            "Total allocated pages: {:?}",
            config_page.get_total_allocated_pages()
        );
    } else {
        println!("Failed to convert to ConfigPage");
    }
    memory.flush()?;
    Ok(())
}
