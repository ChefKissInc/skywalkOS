// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[inline]
pub fn bit_test(bitmap: &mut [u64], index: usize) -> bool {
    (bitmap[index / 64] & (1u64 << (index % 64))) != 0
}

#[inline]
pub fn bit_set(bitmap: &mut [u64], index: usize) {
    bitmap[index / 64] |= 1u64 << (index % 64);
}

#[inline]
pub fn bit_reset(bitmap: &mut [u64], index: usize) {
    bitmap[index / 64] &= !(1u64 << (index % 64));
}
