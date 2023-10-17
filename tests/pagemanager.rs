use memory_manager::memory_manager::MemoryManager;
use memory_manager::pages::config_page::{ConfigPage, MemoryLayout};
use memory_manager::pages::page_manager::PageManager;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{self};
use std::os::unix::prelude::FileExt;

#[test]
// this test should fail, since we don't have enough pages to initialize the page manager
fn test_page_manager_initialization_fail() -> io::Result<()> {
    let filename = "test_page_manager_initialization_fail.bin";
    let num_pages = 1u64;
    let mut memory: MemoryManager = MemoryManager::new(filename, num_pages).unwrap();
    let page_manager = PageManager::new(&mut memory, num_pages);
    match page_manager {
        Ok(_) => {
            let _ = fs::remove_file(filename);
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "it shouldn't possible to initialize the page manager with 1 page",
            ))
        }
        Err(e) => {
            assert_eq!(
                e.to_string(),
                "Error: not enough pages! total_allocated_pages: 1, last_used_page: 1"
            );
            let _ = fs::remove_file(filename);
            Ok(())
        }
    }
}
#[test]
fn test_page_manager_initialization() -> io::Result<()> {
    let filename = "test_page_manager_initialization.bin";

    let num_pages = 2u64;
    let mut memory: MemoryManager = MemoryManager::new(filename, num_pages).unwrap();
    PageManager::new(&mut memory, num_pages).unwrap();
    memory.flush()?;

    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 4096];
    file.read_exact(&mut buffer)?;

    let config_page = ConfigPage { data: &mut buffer };

    let expected_at_0 = MemoryLayout {
        total_allocated_pages: 2,
        version_number: 1,
        last_used_page: 1,
        recycled_pages_list: 1,
        previous_config_page: 0,
        offset: 1,
    };
    let result: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 0).unwrap();

    assert_eq!(expected_at_0, result);

    let expected_at_1 = MemoryLayout {
        total_allocated_pages: 0,
        version_number: 0,
        last_used_page: 0,
        recycled_pages_list: 0,
        previous_config_page: 0,
        offset: 0,
    };

    let result_at_0: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 1).unwrap();

    assert_eq!(expected_at_1, result_at_0);

    let _ = fs::remove_file(filename);

    Ok(())
}
#[test]
fn test_page_manager_consolidate_1() -> io::Result<()> {
    let filename = "test_page_manager_consolidate_1.bin";
    let num_pages = 4u64;
    let mut memory: MemoryManager = MemoryManager::new(filename, num_pages).unwrap();
    let mut page_manager: PageManager<'_> = PageManager::new(&mut memory, num_pages).unwrap();

    page_manager.consolidate_state()?;

    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 4096];
    file.read_exact(&mut buffer)?;

    let config_page = ConfigPage { data: &mut buffer };

    // We increment the version number by 1, because the consolidate_state method increments the version number.
    // the last used page is 3, since we use 1 for the config page, 1 for the new free list page, and the recycled pages list is empty
    // but stored at 3
    // the offset it's also incremented
    let expected_at_0 = MemoryLayout {
        total_allocated_pages: 4,
        version_number: 2,
        last_used_page: 2,
        recycled_pages_list: 1,
        previous_config_page: 0,
        offset: 2,
    };
    let result: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 0).unwrap();

    assert_eq!(expected_at_0, result);

    let expected_at_1 = MemoryLayout {
        total_allocated_pages: 4,
        version_number: 1,
        last_used_page: 1,
        recycled_pages_list: 1,
        previous_config_page: 0,
        offset: 1,
    };

    let result_at_0: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 1).unwrap();

    assert_eq!(expected_at_1, result_at_0);

    let _ = fs::remove_file(filename);

    Ok(())
}

#[test]
fn test_page_manager_consolidate_126_1_page() -> io::Result<()> {
    let num_pages = 129u64;
    let mut memory: MemoryManager =
        MemoryManager::new("test_page_manager_consolidate_10_1_page.bin", num_pages).unwrap();
    let mut page_manager: PageManager<'_> = PageManager::new(&mut memory, num_pages).unwrap();

    for _ in 0..126 {
        page_manager.consolidate_state()?;
    }

    let mut file = File::open("test_page_manager_consolidate_10_1_page.bin")?;
    let mut buffer = [0u8; 4096];
    file.read_exact(&mut buffer)?;

    let config_page = ConfigPage { data: &mut buffer };

    let expected_at_0 = MemoryLayout {
        total_allocated_pages: 129,
        version_number: 127,
        last_used_page: 127,
        recycled_pages_list: 1,
        previous_config_page: 0,
        offset: 127,
    };
    let result: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 0).unwrap();

    assert_eq!(expected_at_0, result);

    for i in 2..127 {
        let expected_at_i = MemoryLayout {
            total_allocated_pages: 129,
            version_number: i,
            last_used_page: i,
            recycled_pages_list: 1,
            previous_config_page: 0,
            offset: i,
        };

        let result_at_i: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, i).unwrap();

        assert_eq!(expected_at_i, result_at_i);
    }

    let _ = fs::remove_file("test_page_manager_consolidate_10_1_page.bin");

    Ok(())
}

#[test]
fn test_page_manager_consolidate_252_1_page() -> io::Result<()> {
    let filename = "test_page_manager_consolidate_252_1_page.bin";
    let num_pages = 259u64;
    let mut memory: MemoryManager = MemoryManager::new(filename, num_pages).unwrap();
    let mut page_manager: PageManager<'_> = PageManager::new(&mut memory, num_pages).unwrap();

    for _ in 0..254 {
        page_manager.consolidate_state()?;
    }

    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 4096];
    let mut buffer_prev = [0u8; 4096];
    file.read_exact(&mut buffer)?;

    let config_page: ConfigPage<'_> = ConfigPage { data: &mut buffer };
    file.read_exact_at(
        &mut buffer_prev,
        4096 * config_page.get_previous_config_page(),
    )?;
    let config_page_prev: ConfigPage<'_> = ConfigPage {
        data: &mut buffer_prev,
    };

    let expected_at_0 = MemoryLayout {
        total_allocated_pages: 259,
        version_number: 255,
        last_used_page: 256,
        recycled_pages_list: 1,
        previous_config_page: 130,
        offset: 128,
    };
    let result: MemoryLayout = MemoryLayout::from_bytes_at(&config_page, 0).unwrap();

    assert_eq!(expected_at_0, result);
    let mut result_at_i: MemoryLayout;
    for i in 2..254 {
        let mut expected_at_i = MemoryLayout {
            total_allocated_pages: 259,
            version_number: i,
            last_used_page: i,
            recycled_pages_list: 1,
            previous_config_page: 0,
            offset: i,
        };

        println!("i: {}", i);

        if i < 128 {
            result_at_i = MemoryLayout::from_bytes_at(&config_page_prev, i).unwrap();
        } else {
            if expected_at_i.version_number >= 129 {
                expected_at_i.last_used_page = expected_at_i.last_used_page + 1;
                expected_at_i.recycled_pages_list = 1;
            }
            expected_at_i.previous_config_page = 130;
            expected_at_i.offset = i % 127;

            result_at_i = MemoryLayout::from_bytes_at(&config_page, i % 127).unwrap();
        }

        assert_eq!(expected_at_i, result_at_i);
    }

    let _ = fs::remove_file(filename);

    Ok(())
}
