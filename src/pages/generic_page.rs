#[derive(Debug, PartialEq)]
pub struct GenericPage<'a> {
    pub data: &'a mut [u8],
}

impl<'a> GenericPage<'a> {
    pub fn from_config_page(data: &'a mut [u8]) -> Self {
        GenericPage { data: data }
    }
}
