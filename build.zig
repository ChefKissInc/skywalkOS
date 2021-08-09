// Copyright (c) 2021 VisualDevelopment. All rights reserved.
const std = @import("std");
const deps = @import("./deps.zig");

pub fn build(b: *std.build.Builder) void {
    const exe = b.addExecutable("Fuse.exec", "src/main.zig");
    exe.setTarget(.{
        .cpu_arch = std.Target.Cpu.Arch.x86_64,
        .os_tag = std.Target.Os.Tag.linux,
        .abi = std.Target.Abi.none,
    });
    exe.setBuildMode(b.standardReleaseOptions());
    exe.setOutputDir("../FWLauncher/Build/Drive");
    deps.addAllTo(exe);
    exe.install();
}
