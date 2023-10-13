use std::fmt;

use std::u64;

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct U48(pub [u8; 6]);

impl Default for U48 {
    fn default() -> Self {
        U48 { 0: [0; 6] }
    }
}

impl From<[u8; 6]> for U48 {
    fn from(arr: [u8; 6]) -> Self {
        U48(arr)
    }
}

impl From<u64> for U48 {
    fn from(int: u64) -> Self {
        assert!(0xFFFFFFFFFFFF >= int, "El número no cabe en 48 bits");
        let bytes = int.to_be_bytes(); // Convertir u64 a [u8; 8]
        U48([bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }
}

impl From<usize> for U48 {
    fn from(int: usize) -> Self {
        assert!(0xFFFFFFFFFFFF >= int, "El número no cabe en 48 bits");
        let bytes = int.to_be_bytes(); // Convertir u64 a [u8; 8]
        U48([bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
    }
}
impl From<U48> for [u8; 6] {
    fn from(six_bytes: U48) -> Self {
        six_bytes.0
    }
}

impl From<U48> for u64 {
    fn from(six_bytes: U48) -> Self {
        let mut bytes = [0u8; 8];
        bytes[2..].copy_from_slice(&six_bytes.0);
        u64::from_be_bytes(bytes)
    }
}
impl From<U48> for usize {
    fn from(six_bytes: U48) -> Self {
        let u64_representation: u64 = six_bytes.into();
        u64_representation as usize
    }
}

impl U48 {
    pub const MAX: u64 = 0xFFFFFFFFFFFF;

    pub fn from_bytes(bytes: [u8; 6]) -> Self {
        U48(bytes)
    }

    pub fn from_bytes_range(bytes: &[u8], start: usize, end: usize) -> Self {
        assert!(end - start == 6, "Range must be exactly 6 bytes");
        assert!(end <= bytes.len(), "Range end out of bounds");

        let mut u48_bytes = [0u8; 6];
        u48_bytes.copy_from_slice(&bytes[start..end]);

        U48(u48_bytes)
    }

    pub fn try_from_u64(num: u64) -> Result<U48, &'static str> {
        if num > U48::MAX {
            Err("The number is too large for U48.")
        } else {
            let bytes = num.to_be_bytes(); // Convert u64 a [u8; 8]
            Ok(U48([
                bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
    }
    // Helper function to copy slices to U48
    pub fn copy_slice_to_u48(slice: &[u8]) -> U48 {
        let mut bytes: [u8; 6] = [0; 6];
        bytes.copy_from_slice(slice);
        U48 { 0: bytes }
    }

    pub fn to_bytes(&self) -> &[u8; 6] {
        &self.0
    }

    pub fn to_usize(&self) -> usize {
        usize::from(*self)
    }
    pub fn to_u64(&self) -> u64 {
        u64::from(*self)
    }
}

impl fmt::Debug for U48 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let int_representation: u64 = (*self).into(); // Convertir a u64
        write!(f, "0x{:012x}", int_representation) // Usar 012x para tener siempre 12 caracteres (6 bytes x 2 caracteres por byte)
    }
}
