// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

pub fn parse_elf(
    mem_mgr: &mut super::mem::MemoryManager,
    buffer: &[u8],
) -> (
    sulphur_dioxide::EntryPoint,
    Vec<sulphur_dioxide::kern_sym::KernSymbol>,
) {
    let elf = goblin::elf::Elf::parse(buffer).expect("Failed to parse kernel elf");

    debug!("{:X?}", elf.header);
    assert!(elf.is_64, "Only ELF64");
    assert_eq!(elf.header.e_machine, goblin::elf::header::EM_X86_64);
    assert!(elf.little_endian, "Only little-endian ELFs");
    assert!(
        elf.entry >= amd64::paging::KERNEL_VIRT_OFFSET,
        "Only higher-half kernels"
    );

    let symbols = elf
        .syms
        .iter()
        .map(|v| sulphur_dioxide::kern_sym::KernSymbol {
            start: v.st_value,
            end: v.st_value + v.st_size,
            name: Box::leak(
                elf.strtab
                    .get_at(v.st_name)
                    .unwrap_or("<unknown>")
                    .to_owned()
                    .into_boxed_str(),
            ),
        })
        .collect();

    debug!("Parsing program headers: ");
    for phdr in elf
        .program_headers
        .iter()
        .filter(|phdr| phdr.p_type == goblin::elf::program_header::PT_LOAD)
    {
        assert!(
            phdr.p_vaddr >= amd64::paging::KERNEL_VIRT_OFFSET,
            "Only higher-half kernels."
        );

        let offset = phdr.p_offset as usize;
        let memsz = phdr.p_memsz as usize;
        let file_size = phdr.p_filesz as usize;
        let src = &buffer[offset..(offset + file_size)];
        let dest = unsafe {
            core::slice::from_raw_parts_mut(
                (phdr.p_vaddr - amd64::paging::KERNEL_VIRT_OFFSET) as *mut u8,
                memsz,
            )
        };
        let npages = (memsz + 0xFFF) / 0x1000;
        debug!(
            "vaddr: {:#X}, paddr: {:#X}, npages: {:#X}",
            phdr.p_vaddr,
            phdr.p_vaddr - amd64::paging::KERNEL_VIRT_OFFSET,
            npages
        );
        assert_eq!(
            unsafe { uefi_services::system_table().as_mut() }
                .boot_services()
                .allocate_pages(
                    uefi::table::boot::AllocateType::Address(
                        (phdr.p_vaddr - amd64::paging::KERNEL_VIRT_OFFSET) as _,
                    ),
                    uefi::table::boot::MemoryType::LOADER_DATA,
                    npages,
                )
                .expect("Failed to load section above. Sections might be misaligned."),
            phdr.p_vaddr - amd64::paging::KERNEL_VIRT_OFFSET
        );

        mem_mgr.allocate((
            phdr.p_vaddr - amd64::paging::KERNEL_VIRT_OFFSET,
            npages as u64,
        ));

        for (a, b) in dest
            .iter_mut()
            .zip(src.iter().chain(core::iter::repeat(&0)))
        {
            *a = *b;
        }
    }

    (
        unsafe { core::mem::transmute(elf.entry as *const ()) },
        symbols,
    )
}
