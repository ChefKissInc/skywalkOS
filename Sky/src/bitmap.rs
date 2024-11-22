// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[inline]
pub const fn bit_test(bitmap: &[u64], index: u64) -> bool {
    let index = index as usize;
    (bitmap[index / 64] & (1u64 << (index % 64))) != 0
}

#[inline]
pub fn bit_set(bitmap: &mut [u64], index: u64) {
    let index = index as usize;
    bitmap[index / 64] |= 1u64 << (index % 64);
}

#[inline]
pub fn bit_reset(bitmap: &mut [u64], index: u64) {
    let index = index as usize;
    bitmap[index / 64] &= !(1u64 << (index % 64));
}
