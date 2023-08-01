// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use std::path::PathBuf;

fn main() {
    let cache = fireworkkit::FKCache::new(
        std::fs::read_dir("../../Extensions")
            .unwrap()
            .filter_map(Result::ok)
            .filter(|v| v.path().is_dir())
            .map(|ent| {
                let info: fireworkkit::FKInfo =
                    ron::from_str(&std::fs::read_to_string(ent.path().join("Info.ron")).unwrap())
                        .unwrap();
                println!("{}", info.identifier);
                let payload = std::fs::read(PathBuf::from("../../target/Extensions").join(
                    format!("{}.exec", info.identifier.split('.').last().unwrap()),
                ))
                .unwrap();
                (info, payload)
            })
            .collect(),
    );
    std::fs::write(
        "../../Drive/System/Extensions.fkcache",
        postcard::to_allocvec(&cache).unwrap(),
    )
    .unwrap();
}
