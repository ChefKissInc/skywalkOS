// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub fn bit_test(bitmap: &mut [u64], index: u64) -> bool {
    let index = index as usize;
    (bitmap[index / 64] & (1u64 << (index % 64))) != 0
}

pub fn bit_set(bitmap: &mut [u64], index: u64) {
    let index = index as usize;
    bitmap[index / 64] |= 1u64 << (index % 64);
}

pub fn bit_reset(bitmap: &mut [u64], index: u64) {
    let index = index as usize;
    bitmap[index / 64] &= !(1u64 << (index % 64));
}
