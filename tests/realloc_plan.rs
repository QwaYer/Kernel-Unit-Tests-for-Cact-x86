//! Unit tests for the pure `krealloc` decision math
//! (`rust_mm/src/alloc/realloc_plan.rs`).

use cact_host_tests::realloc_plan::*;

/// `size_of::<HeapBlock>()` in the kernel today: `magic` + `size` + `is_free` +
/// padding + `next` pointer.
const HDR: u32 = 24;

#[test]
fn align_is_eight() {
    assert_eq!(ALIGN, 8);
}

#[test]
fn required_size_plain() {
    // A plain block's pointer sits exactly one header in.
    assert_eq!(required_block_size(HDR, HDR, 1), 24);
    assert_eq!(required_block_size(HDR, HDR, 8), 24);
    assert_eq!(required_block_size(HDR, HDR, 20), 24);
    // 20 is the largest payload a 24-byte block holds (24 - 4 canary).
    assert_eq!(required_block_size(HDR, HDR, 21), 32);
    assert_eq!(required_block_size(HDR, HDR, 100), 104);
}

#[test]
fn required_size_aligned_offset() {
    // A `kmalloc_aligned` pointer can start 8 bytes into the block, so the same
    // payload needs a bigger block.
    assert_eq!(required_block_size(HDR + 8, HDR, 8), 32);
    assert_eq!(required_block_size(HDR + 8, HDR, 1), 32);
}

#[test]
fn required_size_never_below_off() {
    // A block can never be smaller than the pointer offset inside it.
    assert_eq!(required_block_size(64, HDR, 1), 64);
    assert_eq!(required_block_size(64, HDR, 8), 64);
}

#[test]
fn in_place_when_it_fits() {
    assert_eq!(
        plan(24, HDR, HDR, None, 8),
        Realloc::InPlace { block_size: 24 }
    );
    // want=40 needs align_up(40 + 4) = 48.
    assert_eq!(
        plan(64, HDR, HDR, None, 40),
        Realloc::InPlace { block_size: 48 }
    );
    assert_eq!(
        plan(104, HDR, HDR, None, 100),
        Realloc::InPlace { block_size: 104 }
    );
}

#[test]
fn grow_into_next_free_block() {
    // 24 + header + 56 = 104, exactly what want=100 needs.
    assert_eq!(
        plan(24, HDR, HDR, Some(56), 100),
        Realloc::MergeNext { block_size: 104 }
    );
}

#[test]
fn grow_without_neighbour_moves() {
    assert_eq!(plan(24, HDR, HDR, None, 100), Realloc::Move);
    assert_eq!(plan(24, HDR, HDR, None, 21), Realloc::Move);
}

#[test]
fn grow_neighbour_too_small_moves() {
    // 24 + header + 55 = 103 < 104.
    assert_eq!(plan(24, HDR, HDR, Some(55), 100), Realloc::Move);
}

#[test]
fn exact_boundary_is_in_place_one_byte_short_moves() {
    assert_eq!(
        plan(104, HDR, HDR, None, 100),
        Realloc::InPlace { block_size: 104 }
    );
    assert_eq!(plan(103, HDR, HDR, None, 100), Realloc::Move);
}

#[test]
fn aligned_pointer_needs_the_larger_block() {
    assert_eq!(plan(24, HDR + 8, HDR, None, 8), Realloc::Move);
    assert_eq!(
        plan(32, HDR + 8, HDR, None, 8),
        Realloc::InPlace { block_size: 32 }
    );
}

#[test]
fn should_split_threshold() {
    // Slack must carry a whole header plus a minimal payload (one ALIGN).
    assert!(!should_split(24, 24, HDR));
    assert!(should_split(56, 24, HDR));
    assert!(!should_split(55, 24, HDR));
}

#[test]
fn saturates_huge_request() {
    assert_eq!(
        required_block_size(HDR, HDR, u32::MAX),
        u32::MAX & !(ALIGN - 1)
    );
    assert_eq!(plan(1 << 20, HDR, HDR, None, u32::MAX), Realloc::Move);
}
