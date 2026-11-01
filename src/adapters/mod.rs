//! Adapters: concrete implementations of the ports. This is where all
//! external-world knowledge lives (the `ignore` crate, file formats, disk
//! I/O).

pub mod serializer;
pub mod walker;
