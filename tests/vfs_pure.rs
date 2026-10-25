//! Unit tests for the pure VFS helpers (`rust_vfs/src/vfs_pure.rs`).

use cact_host_tests::vfs_pure::*;

#[test]
fn sizes_are_pinned() {
    assert_eq!(DCACHE_BUCKETS, 128);
    assert_eq!(PARENT_MAX, 256);
}

#[test]
fn bucket_index_is_deterministic_and_in_range() {
    for parent in [0u32, 0x1000, 0xdead_beef, 0xffff_ffff] {
        for hash in [0u32, 1, 5381, 0x1234_5678] {
            let b = bucket_index(parent, hash);
            assert!(b < DCACHE_BUCKETS);
            assert_eq!(b, bucket_index(parent, hash)); // deterministic
        }
    }
    assert_eq!(bucket_index(0, 0), 0);
}

#[test]
fn parent_lookup_finds_and_misses() {
    let pairs = [(0x100usize, 0x1_0000usize), (0x200, 0x2_0000)];
    assert_eq!(parent_lookup(&pairs, 2, 0x100), Some(0x1_0000));
    assert_eq!(parent_lookup(&pairs, 2, 0x200), Some(0x2_0000));
    assert_eq!(parent_lookup(&pairs, 2, 0x999), None);
    // Only the live `count` prefix is searched.
    assert_eq!(parent_lookup(&pairs, 1, 0x200), None);
    assert_eq!(parent_lookup(&pairs, 0, 0x100), None);
    // A count past the slice length is clamped, not UB.
    assert_eq!(parent_lookup(&pairs, 999, 0x999), None);
}

#[test]
fn parent_insert_appends_then_overwrites() {
    let mut pairs: [(usize, usize); 4] = [(0, 0); 4];
    let mut count = 0usize;

    assert!(parent_insert(&mut pairs, &mut count, 0x10, 0x1_0000));
    assert_eq!(count, 1);
    assert!(parent_insert(&mut pairs, &mut count, 0x20, 0x2_0000));
    assert_eq!(count, 2);

    // Same child overwrites in place: returns false, count unchanged.
    assert!(!parent_insert(&mut pairs, &mut count, 0x10, 0x9_0000));
    assert_eq!(count, 2);
    assert_eq!(parent_lookup(&pairs, count, 0x10), Some(0x9_0000));
    assert_eq!(parent_lookup(&pairs, count, 0x20), Some(0x2_0000));
}

#[test]
fn parent_insert_refuses_when_full() {
    let mut pairs: [(usize, usize); 2] = [(0, 0); 2];
    let mut count = 0usize;
    assert!(parent_insert(&mut pairs, &mut count, 1, 10));
    assert!(parent_insert(&mut pairs, &mut count, 2, 20));
    // Full: a new child is dropped (correctness does not depend on caching).
    assert!(!parent_insert(&mut pairs, &mut count, 3, 30));
    assert_eq!(count, 2);
    assert_eq!(parent_lookup(&pairs, count, 3), None);
    // But an existing child can still be updated when full.
    assert!(!parent_insert(&mut pairs, &mut count, 1, 11));
    assert_eq!(parent_lookup(&pairs, count, 1), Some(11));
}
