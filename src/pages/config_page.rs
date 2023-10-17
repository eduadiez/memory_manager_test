use byteorder::ByteOrder;
use byteorder::LittleEndian;
use std::fmt;

// Defining constants to avoid magic numbers
const TOTAL_ALLOCATED_PAGES_BYTES: usize = 6;
const TOTAL_ALLOCATED_PAGES_START: usize = 0; // 6 bytes

const VERSION_NUMBER_BYTES: usize = 5;
const VERSION_NUMBER_START: usize = TOTAL_ALLOCATED_PAGES_START + TOTAL_ALLOCATED_PAGES_BYTES; // 5 bytes

const LAST_USED_PAGE_BYTES: usize = 6;
const LAST_USED_PAGE_START: usize = VERSION_NUMBER_START + VERSION_NUMBER_BYTES; // 6 bytes

const RECYCLED_PAGES_LIST_BYTES: usize = 6;
const RECYCLED_PAGES_LIST_START: usize = LAST_USED_PAGE_START + LAST_USED_PAGE_BYTES; // 6 bytes

const PREVIOUS_CONFIG_PAGE_BYTES: usize = 6;
const PREVIOUS_CONFIG_PAGE_START: usize = RECYCLED_PAGES_LIST_START + RECYCLED_PAGES_LIST_BYTES; // 6 bytes

const OFFSET_BYTES: usize = 3;
const OFFSET_START: usize = PREVIOUS_CONFIG_PAGE_START + PREVIOUS_CONFIG_PAGE_BYTES; // 3 bytes
const OFFSET_END: usize = OFFSET_START + OFFSET_BYTES;

#[derive(Debug, Default, PartialEq)]
#[repr(C)]
pub struct MemoryLayout {
    pub total_allocated_pages: u64,
    pub version_number: u64,
    pub last_used_page: u64,
    pub recycled_pages_list: u64,
    pub previous_config_page: u64,
    pub offset: u64,
}

impl MemoryLayout {
    #![allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.total_allocated_pages.to_le_bytes());
        result.extend_from_slice(&self.version_number.to_le_bytes());
        result.extend_from_slice(&self.last_used_page.to_le_bytes());
        result.extend_from_slice(&self.recycled_pages_list.to_le_bytes());
        result.extend_from_slice(&self.previous_config_page.to_le_bytes());
        result.extend_from_slice(&self.offset.to_le_bytes());
        result
    }
    pub fn from_bytes_at(config_page: &ConfigPage, version: u64) -> Option<Self> {
        Some(Self {
            total_allocated_pages: config_page.get_total_allocated_pages_at(version),
            version_number: config_page.get_version_number_at(version),
            last_used_page: config_page.get_last_used_page_at(version),
            recycled_pages_list: config_page.get_recycled_pages_list_at(version),
            previous_config_page: config_page.get_previous_config_page_at(version),
            offset: config_page.get_offset_at(version),
        })
    }
}

macro_rules! impl_set_get {
    ($name:ident, $start_const:ident, $num_bytes:expr) => {
        paste::paste! {  // Usamos el crate 'paste' para concatenar identificadores

            pub fn [<get_ $name>](&self) -> u64 {
                let mut value = LittleEndian::read_u64(&self.data[$start_const..$start_const + 8]);
                let mask = !0u64 >> (8 * (8 - $num_bytes));
                value &= mask;
                value
            }
            #[allow(dead_code)]
            pub fn [<get_ $name _at>](&self, version: u64) -> u64 {
                if version > self.get_version_number() {
                    panic!("Version number is greater than the current version number");
                }
                let selector = version as usize * OFFSET_END;
                let mut value;
                // we have an issue with the latest element of a [u8;4096] since we only have 4096 bytes, so we need to copy and extend the last element
                if selector + $start_const + 8 > self.data.len() {
                    let mut extended = [0u8; 8]; // Inicializar un array de 8 bytes con 0s
                    extended[..3].copy_from_slice(&self.data[selector + $start_const.. selector + $start_const + $num_bytes]); // Copiar los 3 bytes del array original
                    value = LittleEndian::read_u64(&extended[..8]);
                }else{
                    value = LittleEndian::read_u64(&self.data[selector + $start_const.. selector +$start_const + 8]);

                }

                let mask = !0u64 >> (8 * (8 - $num_bytes));
                value &= mask;
                value
            }

            pub fn [<set_ $name>](&mut self, value: u64) {
                let mut buf = [0u8; 8];
                LittleEndian::write_u64(&mut buf, value);
                self.data[$start_const..$start_const + $num_bytes]
                    .copy_from_slice(&buf[0..$num_bytes]);
            }
        }
    };
}

// TODO: Error handling

#[derive(PartialEq)]
pub struct ConfigPage<'a> {
    pub data: &'a mut [u8],
}

impl<'a> ConfigPage<'a> {
    impl_set_get!(
        total_allocated_pages,
        TOTAL_ALLOCATED_PAGES_START,
        TOTAL_ALLOCATED_PAGES_BYTES
    );
    impl_set_get!(version_number, VERSION_NUMBER_START, VERSION_NUMBER_BYTES);
    impl_set_get!(last_used_page, LAST_USED_PAGE_START, LAST_USED_PAGE_BYTES);
    impl_set_get!(
        recycled_pages_list,
        RECYCLED_PAGES_LIST_START,
        RECYCLED_PAGES_LIST_BYTES
    );
    impl_set_get!(
        previous_config_page,
        PREVIOUS_CONFIG_PAGE_START,
        PREVIOUS_CONFIG_PAGE_BYTES
    );
    impl_set_get!(offset, OFFSET_START, OFFSET_BYTES);

    pub fn copy_header_to_offset(&mut self) {
        let mut bytes = [0u8; OFFSET_END];
        bytes.copy_from_slice(&self.data[0..OFFSET_END]);
        let sel = (self.get_offset() * OFFSET_END as u64) as usize;
        self.data[sel..sel + OFFSET_END].copy_from_slice(&bytes);
    }
    // Method to set the contents of this ConfigPage with the contents of another ConfigPage.
    pub fn copy_config_page(&mut self, config: &ConfigPage) {
        self.data[0..4096].copy_from_slice(&config.data[0..4096]); // Copying the data from config into self.
    }

    pub fn copy_config_page_header(&mut self, config: &ConfigPage) {
        self.data[0..32].copy_from_slice(&config.data[0..32]); // Copying the data from config into self.
    }
}

// Implementing the Debug trait for ConfigPage to enable custom debug formatting.
impl<'a> fmt::Debug for ConfigPage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Custom formatting to display the result of get_total_allocated_pages when formatting ConfigPage for debugging.
        write!(
            f,
            "ConfigPage {{ total_allocated_pages: {:?}, version_number: {:?}, last_used_page: {:?}, recycled_pages_list: {:?}, previous_config_page: {:?}, offset: {:?} }}",
            &self.get_total_allocated_pages(),
            &self.get_version_number(),
            &self.get_last_used_page(),
            &self.get_recycled_pages_list(),
            &self.get_previous_config_page(),
            &self.get_offset()
        )
    }
}
