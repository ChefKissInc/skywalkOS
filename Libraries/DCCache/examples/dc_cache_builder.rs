// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use std::cell::UnsafeCell;

use hashbrown::HashMap;

fn main() {
    let contents = std::fs::read_to_string("DistributionInfo.ron").unwrap();
    let dist_info: dc_cache::DistributionInfo = ron::from_str(&contents).unwrap();
    println!(
        "Creating DriverCore cache for {} v{}",
        dist_info.branding, dist_info.version
    );

    let mut cache = dc_cache::DCCache {
        branding: dist_info.branding,
        version: dist_info.version,
        infos: vec![],
        payloads: HashMap::new(),
    };
    let contents = UnsafeCell::new(Vec::new());
    let payloads = UnsafeCell::new(Vec::new());
    for ent in std::fs::read_dir("DCExtensions")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|v| v.path().is_dir())
    {
        let contents = unsafe { contents.get().as_mut().unwrap() };
        contents.push(std::fs::read_to_string(ent.path().join("Info.ron")).unwrap());

        let info: dc_cache::DCInfo = ron::from_str(contents.last().unwrap()).unwrap();
        let payloads = unsafe { payloads.get().as_mut().unwrap() };
        payloads.push(
            std::fs::read(
                std::path::PathBuf::from("target/DCExtensions").join(format!("{}.exec", info.name)),
            )
            .unwrap(),
        );
        cache
            .payloads
            .insert(info.identifier, payloads.last().unwrap());
        println!(
            "Inserting DriverCore extension {} <{}> v{} to cache",
            info.name, info.identifier, info.version
        );
        cache.infos.push(info);
    }
    std::fs::write(
        "Drive/System/DCExtensions.dccache",
        postcard::to_allocvec(&cache).unwrap(),
    )
    .unwrap();
}
