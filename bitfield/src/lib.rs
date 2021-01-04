// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::bitfield;

pub trait Specifier {
    const BITS: u8;
}

#[inline]
fn mask_bits(start: u8, end: u8) -> u8 {
    let len = (end - start) as u16;
    (((1 << len) - 1) as u8) << start
}

#[inline]
pub fn set_bits(data: &mut u8, start: u8, end: u8, value: &mut u64) {
    let mask = mask_bits(start, end);
    let reversed = reverse_bits(*value, (end - start) as usize) as u8;
    *data &= !mask;
    *data |= mask & (reversed << start);
    *value >>= end - start;
}

#[inline]
pub fn get_bits(data: &u8, start: u8, end: u8, value: &mut u64) {
    let mask = mask_bits(start, end);
    *value <<= end - start;
    *value |= ((data & mask) >> start) as u64;
}

#[inline]
pub fn reverse_bits(value: u64, bits: usize) -> u64 {
    value.reverse_bits() >> (64 - bits)
}

macro_rules! bits(
    ( $($name:ident:$bits:literal),+ $(,)? ) => { $(
        pub struct $name;
        impl Specifier for $name {
            const BITS: u8 = $bits;
        }
    )* };
);

bits!(
    B1 : 1, B2 : 2, B3 : 3, B4 : 4, B5 : 5, B6 : 6, B7 : 7, B8 : 8,
    B9 : 9, B10:10, B11:11, B12:12, B13:13, B14:14, B15:15, B16:16,
    B17:17, B18:18, B19:19, B20:20, B21:21, B22:22, B23:23, B24:24,
    B25:25, B26:26, B27:27, B28:28, B29:29, B30:30, B31:31, B32:32,
    B33:33, B34:34, B35:35, B36:36, B37:37, B38:38, B39:39, B40:40,
    B41:41, B42:42, B43:43, B44:44, B45:45, B46:46, B47:47, B48:48,
    B49:49, B50:50, B51:51, B52:52, B53:53, B54:54, B55:55, B56:56,
    B57:57, B58:58, B59:59, B60:60, B61:61, B62:62, B63:63, B64:64,
);
