// Copyright (c) 2021 VisualDevelopment. All rights reserved.

export var stack_bytes: [16 * 1024]u8 align(16) linksection(".bss") = undefined;
const stack = stack_bytes[0..];

export fn _start() callconv(.Naked) noreturn {
    @call(.{ .stack = stack }, kmain, .{});

    while (true) {}
}

fn kmain() void {}
