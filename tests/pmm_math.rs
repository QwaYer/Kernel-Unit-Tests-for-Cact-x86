//! Unit tests for the pure PMM math (`rust_mm/src/pmm_math.rs`).

use cact_host_tests::pmm_math::*;

#[test]
fn page_and_address_round_trip() {
    assert_eq!(PAGE_SIZE, 4096);
    assert_eq!(page_to_addr(0), 0);
    assert_eq!(page_to_addr(1), 4096);
    assert_eq!(page_to_addr(256), 0x10_0000);
    assert_eq!(addr_to_page(0), 0);
    assert_eq!(addr_to_page(4095), 0);
    assert_eq!(addr_to_page(4096), 1);
    assert_eq!(addr_to_page(0x10_0FFF), 0x100);
    assert_eq!(addr_to_page(page_to_addr(7)), 7);
}

#[test]
fn bitmap_index_and_mask() {
    assert_eq!(bitmap_byte(0), 0);
    assert_eq!(bitmap_byte(7), 0);
    assert_eq!(bitmap_byte(8), 1);
    assert_eq!(bitmap_byte(63), 7);
    assert_eq!(bitmap_mask(0), 0x01);
    assert_eq!(bitmap_mask(1), 0x02);
    assert_eq!(bitmap_mask(7), 0x80);
    assert_eq!(bitmap_mask(8), 0x01);
    // Each index maps to exactly one bit inside its byte.
    for i in 0u32..256 {
        assert_eq!(bitmap_mask(i), 1u8 << (i % 8));
        assert_eq!(bitmap_mask(i).count_ones(), 1);
        assert_eq!(bitmap_byte(i), (i / 8) as usize);
    }
}

#[test]
fn clip_to_limit_edges() {
    assert_eq!(clip_to_limit(0x1000, 0, 0x8000), None); // empty
    assert_eq!(clip_to_limit(0x8000, 0x1000, 0x8000), None); // base at limit
    assert_eq!(clip_to_limit(0x9000, 0x1000, 0x8000), None); // base past limit
    assert_eq!(clip_to_limit(0x1000, 0x1000, 0x8000), Some((0x1000, 0x2000)));
    assert_eq!(clip_to_limit(0x1000, 0x9000, 0x8000), Some((0x1000, 0x8000))); // clipped
    // base+len overflow must saturate, not wrap to a small end.
    assert_eq!(
        clip_to_limit(u64::MAX - 0x100, 0x1000, u64::MAX),
        Some((u64::MAX - 0x100, u64::MAX)),
    );
}

#[test]
fn aligned_page_range_rounds_inward() {
    assert_eq!(aligned_page_range(0x1000, 0x3000), (1, 3));
    // Unaligned start rounds up, unaligned end rounds down.
    assert_eq!(aligned_page_range(0x1000, 0x2001), (1, 2));
    // A range inside a single page yields an empty (first == last) range.
    assert_eq!(aligned_page_range(0x1001, 0x2FFF), (2, 2));
    assert_eq!(aligned_page_range(0, 0x1000), (0, 1));
}

#[test]
fn ram_end_from_candidate_cases() {
    const LIMIT: u64 = 0x1_0000_0000; // 4 GiB
    const RESERVED: u32 = 0x0200_0000;

    assert_eq!(ram_end_from_candidate(0, RESERVED, LIMIT), None); // nothing usable
    assert_eq!(
        ram_end_from_candidate(0x7FFF_E000, RESERVED, LIMIT),
        Some(0x7FFF_E000)
    );
    // Unaligned candidate rounds up.
    assert_eq!(
        ram_end_from_candidate(0x7FFF_E001, RESERVED, LIMIT),
        Some(0x7FFF_F000)
    );
    // At/over the limit clamps one page below it.
    assert_eq!(
        ram_end_from_candidate(LIMIT, RESERVED, LIMIT),
        Some((LIMIT - PAGE_SIZE as u64) as u32)
    );
    // Below the reserved area -> None (caller keeps its default).
    assert_eq!(ram_end_from_candidate(0x1000, RESERVED, LIMIT), None);
}

#[test]
fn mmio_window_base_is_2mib_aligned_and_clear_of_ram() {
    const HOLE: u32 = 0xC000_0000;
    const ALIGN: u32 = 2 * 1024 * 1024;

    let w1 = mmio_window_base(0x7FFD_E000, HOLE); // ram below the hole
    assert_eq!(w1, HOLE);
    assert_eq!(w1 % ALIGN, 0);

    let w2 = mmio_window_base(0xC010_0000, HOLE); // ram above the hole, unaligned
    assert_eq!(w2, 0xC020_0000);
    assert!(w2 >= 0xC010_0000);
    assert_eq!(w2 % ALIGN, 0);

    let w3 = mmio_window_base(0xC100_0000, HOLE); // already aligned
    assert_eq!(w3, 0xC100_0000);
}
