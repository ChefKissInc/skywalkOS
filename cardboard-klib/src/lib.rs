#![no_std]
#![deny(warnings, clippy::cargo, unused_extern_crates)]

pub mod request;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct Message<'a> {
    pub proc_uuid: uuid::Uuid,
    pub data: &'a [u8],
}

impl<'a> Message<'a> {
    #[must_use]
    pub const fn new(proc_uuid: uuid::Uuid, data: &'a [u8]) -> Self {
        Self { proc_uuid, data }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub enum MessageChannelEntry<'a> {
    Occupied(Message<'a>),
    Unoccupied,
}

impl<'a> MessageChannelEntry<'a> {
    pub fn is_unoccupied(&self) -> bool {
        matches!(self, MessageChannelEntry::Unoccupied)
    }
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

    pub fn try_recv(&mut self) -> Option<Message<'a>> {
        if self.data.iter().all(|v| v.is_unoccupied()) {
            unsafe {
                request::KernelRequest::RefreshMessageChannel
                    .send()
                    .unwrap()
            };
        }
        self.data.iter_mut().find_map(|v| {
            if let MessageChannelEntry::Occupied(msg) = v {
                let val = Some(*msg);
                *v = MessageChannelEntry::Unoccupied;
                val
            } else {
                None
            }
        })
    }
}
