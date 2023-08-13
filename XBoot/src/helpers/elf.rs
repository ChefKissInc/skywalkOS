// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use amd64::paging::{KERNEL_VIRT_OFFSET, PAGE_SIZE};

pub fn parse(
    mem_mgr: &mut super::mem::MemoryManager,
    buffer: &[u8],
) -> (
    sulphur_dioxide::EntryPoint,
    Vec<sulphur_dioxide::KernSymbol>,
) {
    let elf = elf::ElfBytes::<elf::endian::LittleEndian>::minimal_parse(buffer).unwrap();

    assert_eq!(elf.ehdr.class, elf::file::Class::ELF64);
    assert_eq!(elf.ehdr.e_machine, elf::abi::EM_X86_64);
    assert!(
        elf.ehdr.e_entry >= KERNEL_VIRT_OFFSET,
        "Only higher-half kernels"
    );

    let symbols = elf
        .symbol_table()
        .unwrap()
        .map(|(symtab, strtab)| {
            symtab
                .iter()
                .map(|v| sulphur_dioxide::KernSymbol {
                    start: v.st_value,
                    end: v.st_value + v.st_size,
                    name: Box::leak(
                        strtab
                            .get(v.st_name as _)
                            .unwrap_or("<unknown>")
                            .to_owned()
                            .into_boxed_str(),
                    ),
                })
                .collect()
        })
        .unwrap_or_default();

    trace!("Parsing program headers: ");
    let bs = unsafe { uefi_services::system_table().as_mut().boot_services() };
    for phdr in elf
        .segments()
        .unwrap()
        .iter()
        .filter(|phdr| phdr.p_type == elf::abi::PT_LOAD)
    {
        assert!(
            phdr.p_vaddr >= KERNEL_VIRT_OFFSET,
            "Only higher-half kernels."
        );

        let offset = phdr.p_offset as usize;
        let memsz = phdr.p_memsz as usize;
        let file_size = phdr.p_filesz as usize;
        let src = &buffer[offset..(offset + file_size)];
        let dest = unsafe {
            core::slice::from_raw_parts_mut((phdr.p_vaddr - KERNEL_VIRT_OFFSET) as *mut u8, memsz)
        };
        let npages = (memsz + 0xFFF) / PAGE_SIZE as usize;
        trace!(
            "vaddr: {:#X}, paddr: {:#X}, npages: {npages:#X}",
            phdr.p_vaddr,
            phdr.p_vaddr - KERNEL_VIRT_OFFSET
        );
        assert_eq!(
            bs.allocate_pages(
                uefi::table::boot::AllocateType::Address((phdr.p_vaddr - KERNEL_VIRT_OFFSET) as _,),
                uefi::table::boot::MemoryType::LOADER_DATA,
                npages,
            )
            .unwrap(),
            phdr.p_vaddr - KERNEL_VIRT_OFFSET
        );

        mem_mgr.allocate((phdr.p_vaddr - KERNEL_VIRT_OFFSET, npages as u64));

        for (a, b) in dest
            .iter_mut()
            .zip(src.iter().chain(core::iter::repeat(&0)))
        {
            *a = *b;
        }
    }

    (
        unsafe { core::mem::transmute(elf.ehdr.e_entry as *const ()) },
        symbols,
    )
}
