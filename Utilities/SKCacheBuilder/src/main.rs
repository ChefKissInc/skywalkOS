// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![deny(warnings, clippy::nursery, unused_extern_crates)]

use std::path::PathBuf;

fn main() {
    let cache = skykit::SKExtensions::new(
        std::fs::read_dir("../../Extensions")
            .unwrap()
            .filter_map(Result::ok)
            .filter(|v| v.path().is_dir())
            .map(|ent| {
                let info: skykit::SKExtension =
                    ron::from_str(&std::fs::read_to_string(ent.path().join("Info.ron")).unwrap())
                        .unwrap();
                println!("{}", info.identifier);
                let payload = std::fs::read(PathBuf::from("../../target/Extensions").join(
                    format!("{}.exec", info.identifier.rsplit('.').next().unwrap()),
                ))
                .unwrap();
                (info, payload)
            })
            .collect(),
    );
    std::fs::write(
        "../../Drive/System/SkyKitExtensions",
        postcard::to_allocvec(&cache).unwrap(),
    )
    .unwrap();
}
