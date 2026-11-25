// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 CactOS Project

//! Host tests for the pure swap-PTE encoding
//! (`rust_mm/src/fault/swap_pte.rs`), `#[path]`-included from the kernel source.

use cact_host_tests::swap_pte::*;

#[test]
fn device_slot_round_trips_and_sets_only_the_swap_marker() {
    for slot in [0u32, 1, 2, 4096, SLOT_MAX] {
        let pte = encode_device(slot);
        assert_eq!(decode(pte), slot);
        assert!(is_swapped(pte));
        assert!(!in_zswap(pte));
        // Only the marker byte is set; the frame field carries the slot.
        assert_eq!(pte & 0xFFF, PAGE_SWAPPED as u64);
    }
}

#[test]
fn zswap_handle_round_trips_and_is_distinguishable() {
    for handle in [0u32, 1, 65535, SLOT_MAX] {
        let pte = encode_zswap(handle);
        assert_eq!(decode(pte), handle);
        assert!(is_swapped(pte));
        assert!(in_zswap(pte));
        // The discriminator is exactly PAGE_ZSWAP on top of the device encoding.
        assert_eq!(pte, encode_device(handle) | PAGE_ZSWAP as u64);
    }
}

#[test]
fn present_pte_is_never_seen_as_paged_out() {
    // Bit 3 doubles as PAGE_PWT in a present entry (MMIO PTEs set it), so a
    // present PTE with bit 3 set must not be mistaken for a swapped one.
    let pte = (PAGE_PRESENT | PAGE_SWAPPED) as u64;
    assert!(!is_swapped(pte));
    assert!(!in_zswap(pte));
    // And the zswap marker on a present entry is likewise not a page-out.
    let pte = (PAGE_PRESENT | PAGE_SWAPPED | PAGE_ZSWAP) as u64;
    assert!(!is_swapped(pte));
    assert!(!in_zswap(pte));
}

#[test]
fn slot_max_is_the_twenty_bit_frame_field() {
    assert_eq!(SLOT_MAX, 0x000F_FFFF);
    assert_eq!(SLOT_SHIFT, 12);
    // A slot at the top of the field must not spill into the marker bits.
    let pte = encode_device(SLOT_MAX);
    assert_eq!(pte & 0xFFF, PAGE_SWAPPED as u64);
    assert_eq!((pte & SLOT_MASK as u64) >> SLOT_SHIFT, SLOT_MAX as u64);
}

#[test]
fn decode_ignores_the_marker_bits() {
    // A zswap entry and its device twin decode to the same slot/handle.
    let handle = 0x12345;
    assert_eq!(decode(encode_zswap(handle)), decode(encode_device(handle)));
    // decode() is a pure mask, so marker bits alone give 0.
    assert_eq!(decode((PAGE_SWAPPED | PAGE_ZSWAP) as u64), 0);
}
