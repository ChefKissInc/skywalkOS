use alloc::boxed::Box;

use log::info;

amd64::impl_pml4!(
    {
        let ret = Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as usize
            - amd64::paging::PHYS_VIRT_OFFSET;
        info!("{:#X?}, {:#X?}", ret, ret + amd64::paging::PHYS_VIRT_OFFSET);
        ret
    },
    amd64::paging::PHYS_VIRT_OFFSET as usize
);
