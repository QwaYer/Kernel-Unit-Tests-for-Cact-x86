//! Unit tests for the pure heap size-class mapping
//! (`rust_mm/src/alloc/heap_class.rs`).

use cact_host_tests::heap_class::*;

#[test]
fn class_count_covers_the_window() {
    // 32 classes reach 2^31, far past the 16 MiB heap window.
    assert_eq!(CLASS_COUNT, 32);
    assert!(class_size(CLASS_COUNT - 1) >= 16 * 1024 * 1024);
}

#[test]
fn small_sizes_land_in_their_power_of_two_class() {
    assert_eq!(class_of(16), 4);
    assert_eq!(class_of(17), 5);
    assert_eq!(class_of(24), 5);
    assert_eq!(class_of(32), 5);
    assert_eq!(class_of(33), 6);
    assert_eq!(class_of(2048), 11);
    assert_eq!(class_of(2049), 12);
}

#[test]
fn class_size_is_at_least_the_size() {
    for s in [16u32, 17, 31, 32, 100, 1000, 4096, 65536, 1 << 24] {
        let c = class_of(s);
        assert!(class_size(c) >= s, "size {s} -> class {c} too small");
        assert!(c > 0 && class_size(c - 1) < s, "size {s} -> class {c} too big");
    }
}

#[test]
fn class_of_is_monotonic() {
    let mut prev = class_of(16);
    for s in 17u32..5000 {
        let c = class_of(s);
        assert!(c >= prev, "class went down at size {s}");
        prev = c;
    }
}

#[test]
fn huge_sizes_clamp_to_the_top_class() {
    assert_eq!(class_of(u32::MAX), CLASS_COUNT - 1);
    assert_eq!(class_of(1 << 31), CLASS_COUNT - 1);
}

#[test]
fn smallest_block_is_class_four() {
    // The heap never makes a block below MIN_FREE = 16 bytes.
    assert_eq!(class_of(16), 4);
    assert_eq!(class_size(4), 16);
}
