//! Unit tests for the pure slab-page ownership decision
//! (`rust_mm/src/alloc/slab_owner.rs`).

use cact_host_tests::slab_owner::{is_slab_header, SLAB_MAGIC};

/// The value a plain heap block's page base carries in its first word
/// (`HEAP_MAGIC`, ffi.rs).
const HEAP_MAGIC: u32 = 0xDEAD_BEEF;

#[test]
fn magic_is_nonzero_and_distinct_from_heap_magic() {
    assert_ne!(SLAB_MAGIC, 0);
    assert_ne!(SLAB_MAGIC, HEAP_MAGIC);
}

#[test]
fn a_live_slab_header_is_owned() {
    assert!(is_slab_header(SLAB_MAGIC, 8, true));
    assert!(is_slab_header(SLAB_MAGIC, 507, true));
    assert!(is_slab_header(SLAB_MAGIC, 1, true));
}

#[test]
fn a_heap_block_page_is_not_a_slab() {
    // A plain kmalloc page base starts with HeapBlock { magic: HEAP_MAGIC, ... }:
    // the magic mismatches, so it is never mistaken for a slab whatever the rest
    // of the header looks like — the old capacity-range guess could match.
    assert!(!is_slab_header(HEAP_MAGIC, 507, true));
    assert!(!is_slab_header(HEAP_MAGIC, 1, true));
    assert!(!is_slab_header(HEAP_MAGIC, 0, false));
}

#[test]
fn wrong_magic_is_never_a_slab() {
    assert!(!is_slab_header(0, 8, true));
    assert!(!is_slab_header(0xDEAD_C0DE, 8, true));
    assert!(!is_slab_header(SLAB_MAGIC ^ 1, 8, true));
}

#[test]
fn zero_capacity_is_not_a_slab() {
    assert!(!is_slab_header(SLAB_MAGIC, 0, true));
}

#[test]
fn missing_cache_pointer_is_not_a_slab() {
    assert!(!is_slab_header(SLAB_MAGIC, 8, false));
    // Even the strongest-looking header needs all three facts to hold.
    assert!(!is_slab_header(SLAB_MAGIC, 508, false));
}
