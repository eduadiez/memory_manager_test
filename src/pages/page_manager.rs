use crate::logger;
use crate::memory_manager;
use crate::memory_manager::MemoryManager;
use crate::pages::config_page::ConfigPage;
use slog::{crit, info};
use std::io::{self, ErrorKind};
use std::mem;

pub struct PageManager<'a> {
    memory: &'a mut MemoryManager,
    config_page: ConfigPage<'a>,
    last_used_page: u64,
    recycled_pages: Vec<u64>,
    recycled_pages_page: u64,
    total_allocated_pages: u64,
}

impl<'a> PageManager<'a> {
    pub fn new(memory: &'a mut MemoryManager, num_pages: u64) -> Result<Self, std::io::Error> {
        let log: &slog::Logger = logger::get_logger();
        let config_page =
            memory.get_page_mut::<ConfigPage>(memory_manager::RESERVED_CONFIG_PAGE_INDEX)?;

        let last_used_page = config_page.get_last_used_page();
        let recycled_pages_page = config_page.get_recycled_pages_list();

        let mut page_manager = PageManager {
            config_page: config_page,
            memory: memory,
            last_used_page: last_used_page,
            recycled_pages: vec![],
            recycled_pages_page: recycled_pages_page,
            total_allocated_pages: num_pages,
        };

        // Check if the memory is initalized
        if page_manager.config_page.get_total_allocated_pages() == 0 {
            info!(log, "Initializing memory...");

            if page_manager.last_used_page != 0 || page_manager.recycled_pages_page != 0 {
                let err_msg = format!(
                    "Database file is corrupted: last_used_page != 0 || recycled_pages_page != 0"
                );
                crit!(log, "{}", &err_msg);
                return Err(io::Error::new(ErrorKind::Other, err_msg));
            }
            page_manager.recycled_pages_page = page_manager.get_free_pages(1).remove(0);
            page_manager.consolidate_state()?;
        }

        info!(log, "Memory initialized: total_allocated_pages: {:?} - last_used_page: {:?} - recycled_pages_page: {:?}",
            page_manager.config_page.get_total_allocated_pages(),
            page_manager.config_page.get_last_used_page(),
            page_manager.config_page.get_recycled_pages_list(),
        );
        Ok(page_manager)
    }

    pub fn get_last_used_page(&self) -> u64 {
        self.last_used_page
    }

    pub fn get_free_pages(&mut self, num: u64) -> Vec<u64> {
        let mut free_pages: Vec<u64> = vec![];
        // TODO: Pending to add the logic to get pages from the recycled pages list
        for i in 0..num {
            free_pages.push(i + self.last_used_page + 1);
        }
        self.last_used_page += num;
        free_pages
    }

    pub fn recyle_pages(&mut self, pages: &mut Vec<u64>) {
        self.recycled_pages.append(pages);
    }

    pub fn get_recyle_pages(&self) -> &Vec<u64> {
        &self.recycled_pages
    }

    pub fn consolidate_state(&mut self) -> Result<(), std::io::Error> {
        let log: &slog::Logger = logger::get_logger();

        // create a temporal config page to copy the data
        let mut free_page = self.get_free_pages(1);
        let next_page = free_page.remove(0);
        let mut config_page_tmp = self.memory.get_page_mut::<ConfigPage>(next_page)?;

        config_page_tmp.set_last_used_page(self.last_used_page);
        config_page_tmp.set_recycled_pages_list(self.recycled_pages_page);
        config_page_tmp.set_total_allocated_pages(self.total_allocated_pages);

        if self.recycled_pages.len() != 0 {
            info!(log, "Recycling pages...");
            // We need to update the recycled pages list
            if self.recycled_pages.len() != 0 {}
        }

        self.config_page.copy_config_page(&config_page_tmp);
        self.memory.flush()?;
        Ok(())
    }
}
