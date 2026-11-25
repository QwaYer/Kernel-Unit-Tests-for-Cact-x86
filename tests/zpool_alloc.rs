// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 CactOS Project

//! Host tests for the compressed-page pool's slot allocator
//! (`rust_mm::comp::zpool_alloc`).  The module is `#[path]`-included from the
//! kernel source, so these exercise exactly what the kernel compiles.

use cact_host_tests::zpool_alloc::{AllocError, SlotAlloc, HANDLE_NONE};

#[test]
fn init_links_every_slot_and_reports_free() {
    let mut links = [0u32; 4];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);

    assert_eq!(a.capacity(), 4);
    assert_eq!(a.live(), 0);
    assert_eq!(a.free_slots(), 4);
    assert_eq!(a.used_bytes(), 0);
    assert_eq!(a.budget_bytes(), 1 << 20);
    assert_eq!(links, [1, 2, 3, HANDLE_NONE]);
}

#[test]
fn alloc_hands_out_slots_in_order_until_full() {
    let mut links = [0u32; 3];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);

    assert_eq!(a.alloc(&mut links, 10), Ok(0));
    assert_eq!(a.alloc(&mut links, 10), Ok(1));
    assert_eq!(a.alloc(&mut links, 10), Ok(2));
    assert_eq!(a.alloc(&mut links, 10), Err(AllocError::NoSlot));
    assert_eq!(a.live(), 3);
    assert_eq!(a.free_slots(), 0);
    assert_eq!(a.used_bytes(), 30);
}

#[test]
fn budget_refusal_leaves_the_pool_untouched() {
    let mut links = [0u32; 4];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 100);

    assert_eq!(a.alloc(&mut links, 60), Ok(0));
    // 60 + 60 > 100: refused, and it must not consume a slot.
    assert_eq!(a.alloc(&mut links, 60), Err(AllocError::NoBudget));
    assert_eq!(a.live(), 1);
    assert_eq!(a.free_slots(), 3);
    assert_eq!(a.used_bytes(), 60);
    // A payload that does fit still succeeds with the head unchanged.
    assert_eq!(a.alloc(&mut links, 40), Ok(1));
    assert_eq!(a.used_bytes(), 100);
}

#[test]
fn free_returns_the_slot_to_the_head_and_credits_the_budget() {
    let mut links = [0u32; 3];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);

    assert_eq!(a.alloc(&mut links, 8), Ok(0));
    assert_eq!(a.alloc(&mut links, 8), Ok(1));
    assert!(a.free(&mut links, 0, 8));
    assert_eq!(a.live(), 1);
    assert_eq!(a.used_bytes(), 8);
    // LIFO: slot 0 goes back to the head, so the next alloc takes it again.
    assert_eq!(a.alloc(&mut links, 8), Ok(0));
    assert_eq!(a.live(), 2);
    assert_eq!(a.used_bytes(), 16);
}

#[test]
fn free_of_an_out_of_range_handle_changes_nothing() {
    let mut links = [0u32; 2];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);
    assert_eq!(a.alloc(&mut links, 4), Ok(0));

    assert!(!a.free(&mut links, 2, 4));
    assert!(!a.free(&mut links, HANDLE_NONE, 4));
    assert_eq!(a.live(), 1);
    assert_eq!(a.used_bytes(), 4);
}

#[test]
fn zero_capacity_pool_always_refuses() {
    let mut links = [0u32; 0];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);

    assert_eq!(a.capacity(), 0);
    assert_eq!(a.free_slots(), 0);
    assert_eq!(a.alloc(&mut links, 1), Err(AllocError::NoSlot));
}

#[test]
fn reuse_after_full_exhaustion_stays_consistent() {
    let mut links = [0u32; 2];
    let mut a = SlotAlloc::new();
    a.init(&mut links, 1 << 20);
    assert_eq!(a.alloc(&mut links, 5), Ok(0));
    assert_eq!(a.alloc(&mut links, 5), Ok(1));
    assert_eq!(a.alloc(&mut links, 5), Err(AllocError::NoSlot));

    a.free(&mut links, 0, 5);
    a.free(&mut links, 1, 5);
    assert_eq!(a.live(), 0);
    assert_eq!(a.used_bytes(), 0);
    assert_eq!(a.free_slots(), 2);
    // The whole list is usable again.
    assert_eq!(a.alloc(&mut links, 5), Ok(1));
    assert_eq!(a.alloc(&mut links, 5), Ok(0));
}
