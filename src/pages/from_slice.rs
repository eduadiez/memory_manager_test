use crate::pages::config_page::ConfigPage;
use crate::pages::free_list_page::FreeListPage;
use crate::pages::generic_page::GenericPage;

// Defining a trait FromSlice with a lifetime parameter 'a.
// This trait specifies a single method, from_slice, which takes a mutable reference to a byte slice
// and returns an instance of the implementing type.
pub trait FromSlice<'a> {
    fn from_slice(data: &'a mut [u8]) -> Self;
}

// Definición de la macro
macro_rules! impl_from_slice {
    ($type:ident, $lifetime:lifetime) => {
        // Implementación del trait FromSlice para el tipo proporcionado
        impl<'a> FromSlice<'a> for $type<$lifetime> {
            fn from_slice(data: &'a mut [u8]) -> Self {
                $type { data: data } // Creando una nueva instancia del tipo proporcionado con la slice de bytes proporcionada
            }
        }
    };
}

// Uso de la macro para generar las implementaciones del trait FromSlice
// para GenericPage y ConfigPage
impl_from_slice!(GenericPage,'a);
impl_from_slice!(ConfigPage,'a);
impl_from_slice!(FreeListPage,'a);
/*
// Implementing the FromSlice trait for the GenericPage type.
// This implementation will allow a GenericPage to be constructed from a mutable byte slice.
impl<'a> FromSlice<'a> for GenericPage<'a> {
    fn from_slice(data: &'a mut [u8]) -> Self {
        GenericPage { data: data } // Creating a new GenericPage instance with the provided byte slice.
    }
}

// Implementing the FromSlice trait for the ConfigPage type.
// This implementation will allow a ConfigPage to be constructed from a mutable byte slice.
impl<'a> FromSlice<'a> for ConfigPage<'a> {
    fn from_slice(data: &'a mut [u8]) -> Self {
        ConfigPage { data: data } // Creating a new ConfigPage instance with the provided byte slice.
    }
}
 */
