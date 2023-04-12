// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

fn main() {
    println!("Creating TungstenKit cache");

    let mut cache = tungstenkit::TKCache::default();

    for ent in std::fs::read_dir("Extensions")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|v| v.path().is_dir())
    {
        let info: tungstenkit::TKInfo =
            ron::from_str(&std::fs::read_to_string(ent.path().join("Info.ron")).unwrap()).unwrap();
        println!(
            "Inserting TungstenKit extension {} <{}> v{} to cache",
            info.name, info.identifier, info.version
        );
        let payload = std::fs::read(
            std::path::PathBuf::from("target/Extensions").join(format!("{}.exec", info.name)),
        )
        .unwrap();
        cache.0.push((info, payload));
    }
    std::fs::write(
        "Drive/System/Extensions.tkcache",
        postcard::to_allocvec(&cache).unwrap(),
    )
    .unwrap();
}
