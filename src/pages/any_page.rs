use crate::pages::config_page::ConfigPage;
use crate::pages::free_list_page::FreeListPage;
use crate::pages::generic_page::GenericPage;

// Enum definition for AnyPage, which can hold either a GenericPage or a ConfigPage.
pub enum AnyPage<'a> {
    Generic(GenericPage<'a>),   // Variant for holding a GenericPage.
    Config(ConfigPage<'a>),     // Variant for holding a ConfigPage.
    FreeList(FreeListPage<'a>), // Variant for holding a FreeListPage.
}

// Implementing methods for AnyPage enum.
impl<'a> AnyPage<'a> {
    // Method to attempt converting AnyPage to ConfigPage.
    // If AnyPage is Generic, it converts it to ConfigPage, otherwise returns None.
    pub fn to_config_page(self) -> Option<ConfigPage<'a>> {
        if let AnyPage::Generic(page) = self {
            Some(ConfigPage::from_generic_page(page))
        } else {
            None
        }
    }

    // Method to attempt converting AnyPage to GenericPage.
    // If AnyPage is Config, it converts it to GenericPage, otherwise returns None.
    pub fn to_generic_page(self) -> Option<GenericPage<'a>> {
        if let AnyPage::Config(page) = self {
            Some(GenericPage::from_config_page(page.data))
        } else {
            None
        }
    }
}
