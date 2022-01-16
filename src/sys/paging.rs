/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use alloc::boxed::Box;

amd64::impl_pml4!(
    Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as usize
        - amd64::paging::PHYS_VIRT_OFFSET,
    amd64::paging::PHYS_VIRT_OFFSET as usize
);
