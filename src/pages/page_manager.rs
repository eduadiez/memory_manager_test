use crate::logger;
use crate::memory_manager;
use crate::memory_manager::MemoryManager;
use crate::pages::config_page::ConfigPage;
use crate::pages::free_list_page::FreeListPage;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use slog::{crit, debug, info};
use std::io::{self, ErrorKind};
pub struct PageManager<'a> {
    memory: &'a mut MemoryManager,
    pub config_page: ConfigPage<'a>,
    pub last_used_page: u64,
    pub recycled_pages: Vec<u64>,
    pub recycled_pages_page: u64,
    pub total_allocated_pages: u64,
    pub pending_recycled: Vec<u64>,
}

impl<'a> PageManager<'a> {
    pub fn new(memory: &'a mut MemoryManager, num_pages: u64) -> Result<Self, std::io::Error> {
        let log: &slog::Logger = logger::get_logger();

        let config_page =
            memory.get_page_mut::<ConfigPage>(memory_manager::RESERVED_CONFIG_PAGE_INDEX)?;

        // Read the config page
        info!(log, "Loading config page...");
        let last_used_page = config_page.get_last_used_page();
        let recycled_pages_page = config_page.get_recycled_pages_list();
        let num_pages_config = config_page.get_total_allocated_pages();

        let mut page_manager = PageManager {
            config_page: config_page,
            memory: memory,
            last_used_page: last_used_page,
            recycled_pages: vec![],
            recycled_pages_page: recycled_pages_page,
            total_allocated_pages: num_pages_config,
            pending_recycled: vec![],
        };

        // Check if the memory is initalized
        if num_pages_config == 0 {
            info!(log, "Initializing memory...");

            if page_manager.last_used_page != 0 || page_manager.recycled_pages_page != 0 {
                let err_msg = format!(
                    "Database file is corrupted: last_used_page != 0 || recycled_pages_page != 0"
                );
                crit!(log, "{}", &err_msg);
                return Err(io::Error::new(ErrorKind::Other, err_msg));
            }
            page_manager.total_allocated_pages = num_pages;
            page_manager.recycled_pages_page = page_manager.get_free_pages(1, true)?.remove(0);
            page_manager.consolidate_state_initial()?;
        }

        page_manager.recycled_pages = page_manager
            .memory
            .get_page_mut::<FreeListPage>(page_manager.recycled_pages_page)?
            .get_recycled_pages_list()?;

        debug!(log, "{:?} ", page_manager.config_page);
        Ok(page_manager)
    }

    #[allow(dead_code)]
    pub fn recyle_pages(&mut self, pending: &mut Vec<u64>) {
        self.pending_recycled.append(pending);
    }

    pub fn get_free_pages(
        &mut self,
        num: u64,
        reuse_pages: bool,
    ) -> Result<Vec<u64>, std::io::Error> {
        let mut free_pages: Vec<u64> = vec![];

        if reuse_pages {
            // We use recycled pages first
            if self.recycled_pages.len() != 0 {
                info!(logger::get_logger(), "Using recycled pages...");

                if num <= self.recycled_pages.len() as u64 {
                    free_pages = self.recycled_pages.drain(0..num as usize).collect();
                    debug!(
                        logger::get_logger(),
                        "Recycled pages used: {:?}", free_pages
                    );
                    return Ok(free_pages);
                } else {
                    info!(
                        logger::get_logger(),
                        "Trying to load more recycled pages from memory..."
                    );

                    let current_recycled_pages_page: FreeListPage<'_> = self
                        .memory
                        .get_page_mut::<FreeListPage>(self.recycled_pages_page)
                        .unwrap();

                    // We need to load more free pages
                    while num < self.recycled_pages.len() as u64 {
                        if current_recycled_pages_page.get_free_list_page_next() != 0 {
                            let previous_recycled_pages_page = self
                                .memory
                                .get_page_mut::<FreeListPage>(
                                    current_recycled_pages_page.get_free_list_page_next(),
                                )
                                .unwrap();
                            if self.recycled_pages.len() != 0 {
                                let err_msg = format!(
                                    "Error loading recycled pages: self.recycled_pages.len() != 0"
                                );
                                crit!(logger::get_logger(), "{}", &err_msg);
                                return Err(io::Error::new(ErrorKind::Other, err_msg));
                            } else {
                                self.recycled_pages = previous_recycled_pages_page
                                    .get_recycled_pages_list()
                                    .unwrap();

                                self.recycled_pages_page =
                                    current_recycled_pages_page.get_free_list_page_next();
                            }
                        } else {
                            info!(
                                logger::get_logger(),
                                "Not enough recycled pages, using new pages..."
                            );
                            break;
                        }
                    }
                    if num < self.recycled_pages.len() as u64 {
                        free_pages = self.recycled_pages.drain(0..num as usize).collect();
                        debug!(
                            logger::get_logger(),
                            "Recycled pages used: {:?}", free_pages
                        );
                        return Ok(free_pages);
                    } else {
                        free_pages = self
                            .recycled_pages
                            .drain(0..self.recycled_pages.len())
                            .collect();
                        info!(
                            logger::get_logger(),
                            "Not enough recycled pages, using new pages..."
                        );
                    }
                }
            }
        }

        for _ in free_pages.len() as u64..num {
            self.last_used_page += 1;
            if self.last_used_page >= self.total_allocated_pages {
                let err_msg = format!(
                    "Error: not enough pages! total_allocated_pages: {}, last_used_page: {}",
                    self.total_allocated_pages, self.last_used_page
                );
                crit!(logger::get_logger(), "{}", &err_msg);
                return Err(io::Error::new(ErrorKind::Other, err_msg));
            }
            free_pages.push(self.last_used_page);
        }

        Ok(free_pages)
    }

    pub fn consolidate_state_initial(&mut self) -> Result<(), std::io::Error> {
        self.config_page
            .set_total_allocated_pages(self.total_allocated_pages);
        self.config_page.set_version_number(1);

        self.config_page.set_last_used_page(self.last_used_page);

        self.config_page
            .set_recycled_pages_list(self.recycled_pages_page);
        self.config_page.set_previous_config_page(0);
        self.config_page.set_offset(1);

        self.memory.flush()?;

        Ok(())
    }

    pub fn get_free_list_page_at(&self, version: u64) -> Result<Vec<u64>, std::io::Error> {
        if version > self.config_page.get_version_number() {
            let err_msg = format!("Error: version > self.config_page.get_version_number()");
            crit!(logger::get_logger(), "{}", &err_msg);
            return Err(io::Error::new(ErrorKind::Other, err_msg));
        }
        let recycled_pages_list = self.config_page.get_recycled_pages_list_at(version);
        debug!(
            logger::get_logger(),
            "Recycled pages list at version {}: {:?}", version, recycled_pages_list
        );

        let version_recycled_pages_page = self
            .memory
            .get_page_mut::<FreeListPage>(recycled_pages_list)
            .unwrap();

        Ok(version_recycled_pages_page
            .get_recycled_pages_list()
            .unwrap())
    }

    pub fn consolidate_state(&mut self) -> Result<(), std::io::Error> {
        let log: &slog::Logger = logger::get_logger();

        // Check if we need to store a new free list page
        let actual_recycled_pages_page: FreeListPage<'_> = self
            .memory
            .get_page_mut::<FreeListPage>(self.recycled_pages_page)?;
        let actual_recycled_pages = actual_recycled_pages_page.get_recycled_pages_list()?;

        // create a temporal config page to copy the data
        let next_page_config = self.get_free_pages(1, true)?.remove(0);
        let mut config_page_tmp = self.memory.get_page_mut::<ConfigPage>(next_page_config)?;

        // If the offset is 127 = (4096/32 -1 ) we don't have space to store the next header
        // we create a copy of the config_page and we link to the current one
        if self.config_page.get_offset() > 127 {
            let next_page_config_copy = self.get_free_pages(1, true)?.remove(0);
            let mut config_page_copy = self
                .memory
                .get_page_mut::<ConfigPage>(next_page_config_copy)?;
            config_page_copy.copy_config_page(&self.config_page);
            config_page_tmp.copy_config_page_header(&self.config_page);
            config_page_tmp.set_offset(1);
            config_page_tmp.set_previous_config_page(next_page_config_copy);
            config_page_tmp.copy_header_to_offset();
            config_page_tmp.set_offset(config_page_tmp.get_offset() + 1);
            config_page_tmp.set_version_number(self.config_page.get_version_number() + 1);
        } else {
            // copy the data from the current config page to the temporal one
            config_page_tmp.copy_config_page(&self.config_page);

            config_page_tmp.copy_header_to_offset();
            config_page_tmp.set_offset(self.config_page.get_offset() + 1);
            config_page_tmp.set_version_number(self.config_page.get_version_number() + 1);
        }

        // We need to compute how many pages we need to store the recycled pages, taking into account that we're going to add one extra page
        info!(
            log,
            "Recycling {} pages...",
            self.recycled_pages.len() + self.pending_recycled.len() + 1
        );
        let num_chunks = (self.recycled_pages.len() + self.pending_recycled.len() + 1 + 510) / 511;
        info!(log, "We need {} pages", num_chunks);

        // we added to the pending recycled pages list the next page config since we can reuse it when the process is done
        self.pending_recycled
            .append(&mut [next_page_config].to_vec());

        // We combine the recycled pages with the pending recycled pages mainly because we need to store the recycled pages
        //in the next config page and  for that we need to wait until all the pages needes are reserved
        self.recycled_pages.append(&mut self.pending_recycled);
        self.pending_recycled = vec![];

        debug!(log, "Recycled pages: {:?}", self.recycled_pages);
        // we only create a new page if we need to store the recycled pages
        // TODO: Probably we can remove this check, although we are saving pages
        if actual_recycled_pages != self.recycled_pages {
            let chunk_pages = self.get_free_pages(num_chunks as u64, true)?;
            for (i, chunk) in self.recycled_pages.chunks(510).enumerate() {
                let actual_recycled_pages_page: FreeListPage<'_> = self
                    .memory
                    .get_page_mut::<FreeListPage>(self.recycled_pages_page)?;

                debug!(
                    log,
                    "Previous recycled pages page: {:?}",
                    actual_recycled_pages_page.get_free_list_page_next()
                );

                let mut current_recycled_pages_page: FreeListPage<'_> =
                    self.memory.get_page_mut::<FreeListPage>(chunk_pages[i])?;

                current_recycled_pages_page
                    .set_free_list_page_next(actual_recycled_pages_page.get_free_list_page_next());

                let mut chunk_vec = chunk.to_vec(); // Resize to have length of 510, fill with zeros
                chunk_vec.resize(510, 0u64);

                let mut bytes = Vec::with_capacity(chunk_vec.len() * 8); // 8 bytes per u64
                for &value in &chunk_vec {
                    let mut buffer = [0u8; 8];
                    LittleEndian::write_u64(&mut buffer, value);
                    bytes.extend_from_slice(&buffer);
                }

                current_recycled_pages_page.set_free_list_page_data_slice(&bytes);
                self.recycled_pages_page = chunk_pages[i];
            }
        }

        config_page_tmp.set_last_used_page(self.last_used_page);
        config_page_tmp.set_recycled_pages_list(self.recycled_pages_page);
        config_page_tmp.set_total_allocated_pages(self.total_allocated_pages);

        // copy the data from the temporal config page to the current one
        self.config_page.copy_config_page(&config_page_tmp);
        self.memory.flush()?;

        Ok(())
    }
}
