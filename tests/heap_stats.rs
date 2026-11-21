//! Unit tests for the pure heap-stats aggregation
//! (`rust_mm/src/alloc/heap_stats.rs`).

use cact_host_tests::heap_stats::HeapAggregate;

#[test]
fn empty_aggregate_is_all_zero() {
    let a = HeapAggregate::new();
    assert_eq!(
        a,
        HeapAggregate {
            free_bytes: 0,
            used_bytes: 0,
            free_blocks: 0,
            total_blocks: 0,
            largest_free: 0,
        }
    );
}

#[test]
fn splits_free_and_used_totals() {
    let mut a = HeapAggregate::new();
    a.add(100, true);
    a.add(40, false);
    a.add(60, true);
    assert_eq!(a.free_bytes, 160);
    assert_eq!(a.free_blocks, 2);
    assert_eq!(a.used_bytes, 40);
    assert_eq!(a.total_blocks, 3);
}

#[test]
fn largest_tracks_the_biggest_free_block_only() {
    let mut a = HeapAggregate::new();
    a.add(16, true);
    a.add(4096, false); // in-use: must not become largest_free
    a.add(512, true);
    a.add(64, true);
    assert_eq!(a.largest_free, 512);
}

#[test]
fn largest_free_is_zero_without_free_blocks() {
    let mut a = HeapAggregate::new();
    a.add(1024, false);
    a.add(8, false);
    assert_eq!(a.largest_free, 0);
    assert_eq!(a.free_blocks, 0);
    assert_eq!(a.total_blocks, 2);
}

#[test]
fn totals_saturate_instead_of_wrapping() {
    let mut a = HeapAggregate::new();
    a.add(u32::MAX, true);
    a.add(4096, true);
    a.add(u32::MAX, false);
    a.add(4096, false);
    assert_eq!(a.free_bytes, u32::MAX);
    assert_eq!(a.used_bytes, u32::MAX);
    assert_eq!(a.free_blocks, 2);
    assert_eq!(a.total_blocks, 4);
}

#[test]
fn largest_free_equal_sizes_stay_put() {
    let mut a = HeapAggregate::new();
    a.add(256, true);
    a.add(256, true);
    assert_eq!(a.largest_free, 256);
    assert_eq!(a.free_bytes, 512);
    assert_eq!(a.free_blocks, 2);
}
