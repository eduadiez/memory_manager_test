use criterion::{criterion_group, criterion_main, BenchmarkId, Throughput};

extern crate memory_manager;
use memory_manager::memory_manager::MemoryManager;
use memory_manager::pages::config_page::ConfigPage;
use memory_manager::pages::free_list_page::FreeListPage;
use memory_manager::pages::Page;
use memory_manager::u48::U48;
use rand::Rng;
use std::any;
use std::cell::RefCell;

fn memory_manager_bench(c: &mut criterion::Criterion) {
    let num_pages: u64 = 33_554_432u64;
    let mem: MemoryManager = MemoryManager::new("bench.bin", num_pages).unwrap();

    let mut rng = rand::thread_rng();
    let pages = (0..20000)
        .map(|_| {
            let random_slice: Vec<u8> = (0..4096).map(|_| rng.gen::<u8>()).collect();
            Page {
                data: random_slice.as_slice().try_into().unwrap(),
            }
        })
        .collect::<Vec<_>>();

    // Envuelve pages en un RefCell
    let pages_ref = RefCell::new(pages);

    let random_slice: Vec<u64> = (0..20000).map(|_| rng.gen_range(0..=33_554_431)).collect();

    let throughput = Throughput::Bytes(std::mem::size_of::<Page>() as u64 * 20000);

    let mut group = c.benchmark_group("memory_manager");
    group.throughput(throughput);

    group.bench_with_input(
        BenchmarkId::new("memory_manager_singleton_benchmark", 100),
        &pages_ref,
        |b, data| {
            // Obtén un préstamo mutable de pages_ref
            let mut pages_borrowed = data.borrow_mut();
            b.iter(|| {
                for i in 0..random_slice.len() {
                    let mut any_page: ConfigPage<'_> = mem.get_page_mut(random_slice[i]).unwrap();
                    any_page.data[0] = any_page.data[3] + i as u8;
                }

                /*               for i in 0..random_slice.len() {
                    let mut any_page: ConfigPage<'_> = mem.get_page_mut(random_slice[i]).unwrap();
                    let temp_config_page = ConfigPage {
                        data: &mut pages_borrowed[i as usize].data,
                    };
                    any_page.copy_config_page(&temp_config_page);
                }
                mem.flush().unwrap();*/
            })
        },
    );

    group.finish();
}

fn memory_manager_read_free_list_points_bench(c: &mut criterion::Criterion) {
    let num_pages: u64 = 8000000u64;
    let memory: MemoryManager = MemoryManager::new("bench_2.bin", num_pages).unwrap();

    // We need to initialize the free pages list
    let mut free_list_page = memory.get_page_mut::<FreeListPage>(1).unwrap();

    free_list_page
        .init_free_list_pages(&memory, num_pages)
        .unwrap();

    memory.flush().unwrap();
    
    let get_free_pages_list_ptr_len =
        FreeListPage::get_free_pages_list_ptr(U48::from(1u64), &memory)
            .unwrap()
            .len();
    println!(
        "get_free_pages_list_ptr size: {:?}",
        get_free_pages_list_ptr_len
    );

    let mut rng = rand::thread_rng();
    let pages = (0..num_pages)
        .map(|_| {
            let random_slice: Vec<u8> = (0..4096).map(|_| rng.gen::<u8>()).collect();
            Page {
                data: random_slice.as_slice().try_into().unwrap(),
            }
        })
        .collect::<Vec<_>>();

    // Envuelve pages en un RefCell
    let pages_ref = RefCell::new(pages);
    let throughput =
        Throughput::Bytes(get_free_pages_list_ptr_len as u64 * std::mem::size_of::<u64>() as u64);
    let mut group = c.benchmark_group("memory_manager");

    group.throughput(throughput);

    group.bench_with_input(
        BenchmarkId::new("memory_manager_read_free_list_points_bench", 1),
        &pages_ref,
        |b, data| {
            b.iter(|| {
                FreeListPage::get_free_pages_list_ptr(U48::from(1u64), &memory).unwrap();
            })
        },
    );
}

criterion_group!(
    benches,
    memory_manager_bench,
    /* memory_manager_read_free_list_points_bench */
);
criterion_main!(benches);

pub fn init_memory() {}

/*
fn memory_manager_pages(c: &mut criterion::Criterion) {
    let mem: MemoryManager = memory_manager::memory_manager::MemoryManager::new("test.bin", 20000);

    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..81920000).map(|_| rng.gen()).collect();

    let config_page: ConfigPage = ConfigPage {
        header: ConfigHeader {
            total_allocated_pages: 20000u64.into(),
            free_pages_pointer: 0u64.into(),
            empty_0: 0,
            empty_1: 0,
        },
        free_pages: [0u64.into(); 680],
    };

    let throughput = Throughput::Bytes(std::mem::size_of::<ConfigPage>() as u64);

    let mut group = c.benchmark_group("memory_manager");
    group.throughput(throughput);

    group.bench_with_input(
        BenchmarkId::new("memory_manager_singleton_benchmark", data.len()),
        &data,
        |b, _data| {
            b.iter(|| {
                *mem.get_page_mut(0) = *config_page.as_page();
                mem.flush();
            })
        },
    );

    group.finish();
}
*/
