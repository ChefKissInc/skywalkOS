// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use std::cell::UnsafeCell;

use hashbrown::HashMap;

fn main() {
    println!("Creating TungstenKit cache");

    let mut cache = tungstenkit::TKCache {
        infos: vec![],
        payloads: HashMap::new(),
    };
    let contents = UnsafeCell::new(Vec::new());
    let payloads = UnsafeCell::new(Vec::new());
    for ent in std::fs::read_dir("Extensions")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|v| v.path().is_dir())
    {
        let contents = unsafe { &mut *contents.get() };
        contents.push(std::fs::read_to_string(ent.path().join("Info.ron")).unwrap());

        let info: tungstenkit::TKInfo = ron::from_str(contents.last().unwrap()).unwrap();
        println!(
            "Inserting TungstenKit extension {} <{}> v{} to cache",
            info.name, info.identifier, info.version
        );
        let payloads = unsafe { &mut *payloads.get() };
        payloads.push(
            std::fs::read(
                std::path::PathBuf::from("target/Extensions").join(format!("{}.exec", info.name)),
            )
            .unwrap(),
        );
        cache
            .payloads
            .insert(info.identifier, payloads.last().unwrap());
        cache.infos.push(info);
    }
    std::fs::write(
        "Drive/System/Extensions.dccache",
        postcard::to_allocvec(&cache).unwrap(),
    )
    .unwrap();
}
