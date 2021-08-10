// Copyright (c) 2021 VisualDevelopment. All rights reserved.

const std = @import("std");
const deps = @import("./deps.zig");

pub fn build(b: *std.build.Builder) void {
    const exe = b.addExecutable("Fuse.exec", "src/main.zig");
    var disabled_features = std.Target.Cpu.Feature.Set.empty;
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse2));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse3));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.ssse3));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse4_1));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse4_2));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.mmx));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.sse4a));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.avx));
    disabled_features.addFeature(@enumToInt(std.Target.x86.Feature.avx2));
    var features = std.Target.Cpu.Feature.Set.empty;
    features.addFeature(@enumToInt(std.Target.x86.Feature.soft_float));

    exe.setTarget(.{
        .cpu_arch = .x86_64,
        .os_tag = .freestanding,
        .abi = .none,
        .cpu_model = .{
            .explicit = std.Target.Cpu.Model.generic(std.Target.Cpu.Arch.x86_64),
        },
        .cpu_features_add = features,
        .cpu_features_sub = disabled_features,
    });
    exe.code_model = .kernel;
    exe.pie = false;
    exe.force_pic = false;
    exe.setLinkerScriptPath(.{ .path = "linker.ld" });
    exe.setBuildMode(b.standardReleaseOptions());
    exe.setOutputDir("../FWLauncher/Build/Drive");
    deps.addAllTo(exe);
    exe.install();
}
