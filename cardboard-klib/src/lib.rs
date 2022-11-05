#![no_std]
#![deny(warnings, clippy::cargo, unused_extern_crates)]

pub mod request;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub enum MessageChannelEntry<'a> {
    Occupied {
        source_process: uuid::Uuid,
        data: &'a [u8],
    },
    Unoccupied,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct MessageChannel<'a> {
    pub data: [MessageChannelEntry<'a>; 64],
}

impl<'a> MessageChannel<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [MessageChannelEntry::Unoccupied; 64],
        }
    }
}
