//! Unit tests for the pure munmap range math (`rust_mm/src/munmap_plan.rs`).

use cact_host_tests::munmap_plan::*;

/// Flatten the classifier to `(lo, hi, kind)` for compact assertions.
fn slice(base: u32, len: u32, addr: u32, end: u32) -> Option<(u32, u32, MunmapClip)> {
    clip_region(base, len, addr, end).map(|s| (s.lo, s.hi, s.kind))
}

#[test]
fn no_overlap_returns_none() {
    // Region strictly before the request.
    assert_eq!(slice(0x1000, 0x1000, 0x3000, 0x4000), None);
    // Region strictly after the request.
    assert_eq!(slice(0x5000, 0x1000, 0x1000, 0x2000), None);
    // Region ends exactly where the request starts (shared end is exclusive).
    assert_eq!(slice(0x1000, 0x1000, 0x2000, 0x3000), None);
    // Region starts exactly where the request ends.
    assert_eq!(slice(0x2000, 0x1000, 0x1000, 0x2000), None);
}

#[test]
fn whole_region() {
    assert_eq!(
        slice(0x1000, 0x4000, 0x1000, 0x5000),
        Some((0x1000, 0x5000, MunmapClip::Whole))
    );
    // A request wider than the region still clips to the whole region.
    assert_eq!(
        slice(0x1000, 0x4000, 0x0000, 0x9000),
        Some((0x1000, 0x5000, MunmapClip::Whole))
    );
}

#[test]
fn front_trim() {
    assert_eq!(
        slice(0x1000, 0x4000, 0x1000, 0x2000),
        Some((0x1000, 0x2000, MunmapClip::Front))
    );
}

#[test]
fn back_trim() {
    assert_eq!(
        slice(0x1000, 0x4000, 0x4000, 0x5000),
        Some((0x4000, 0x5000, MunmapClip::Back))
    );
    // A request that runs past the region's end is still just the back.
    assert_eq!(
        slice(0x1000, 0x4000, 0x4000, 0x9000),
        Some((0x4000, 0x5000, MunmapClip::Back))
    );
}

#[test]
fn middle_split() {
    assert_eq!(
        slice(0x1000, 0x4000, 0x2000, 0x3000),
        Some((0x2000, 0x3000, MunmapClip::Middle))
    );
    // Two pages carved out of the middle of a six-page region.
    assert_eq!(
        slice(0x1000, 0x6000, 0x3000, 0x5000),
        Some((0x3000, 0x5000, MunmapClip::Middle))
    );
}

#[test]
fn single_page_region() {
    // The whole (one-page) region.
    assert_eq!(
        slice(0x1000, 0x1000, 0x1000, 0x2000),
        Some((0x1000, 0x2000, MunmapClip::Whole))
    );
    // A sub-page-shaped slice from the back.
    assert_eq!(
        slice(0x1000, 0x1000, 0x1800, 0x2000),
        Some((0x1800, 0x2000, MunmapClip::Back))
    );
}

#[test]
fn only_middle_needs_a_slot() {
    assert!(!MunmapClip::Whole.needs_slot());
    assert!(!MunmapClip::Front.needs_slot());
    assert!(!MunmapClip::Back.needs_slot());
    assert!(MunmapClip::Middle.needs_slot());
}

#[test]
fn high_region_does_not_wrap() {
    // A region near the top of the user window: `base + len` must stay exact.
    let base = 0xBF00_0000u32;
    let len = 0x0080_0000u32;
    assert_eq!(
        clip_region(base, len, base + 0x1000, base + 0x2000),
        Some(MunmapSlice {
            lo: base + 0x1000,
            hi: base + 0x2000,
            kind: MunmapClip::Middle,
        })
    );
    assert_eq!(
        clip_region(base, len, base, base + len),
        Some(MunmapSlice {
            lo: base,
            hi: base + len,
            kind: MunmapClip::Whole,
        })
    );
}
