// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

use uefi::{
    prelude::*,
    proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, FileType},
    CStr16,
};

pub fn open_esp(image: Handle) -> Directory {
    unsafe {
        let mut fs = uefi_services::system_table()
            .as_mut()
            .boot_services()
            .get_image_file_system(image)
            .unwrap();

        fs.open_volume().unwrap()
    }
}

pub fn load(
    esp: &mut Directory,
    path: &CStr16,
    mode: FileMode,
    attributes: FileAttribute,
) -> Vec<u8> {
    let FileType::Regular(mut file) = esp
        .open(path, mode, attributes)
        .unwrap()
        .into_type()
        .unwrap()
    else {
        panic!();
    };

    let mut buffer = vec![0; file.get_boxed_info::<FileInfo>().unwrap().file_size() as _];

    file.read(&mut buffer).unwrap();
    file.close();

    buffer
}
