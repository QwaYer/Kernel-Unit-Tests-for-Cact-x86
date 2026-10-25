//! Unit tests for the pure sibling-aware placement policy
//! (`sched/src/placement.rs`).

use cact_host_tests::placement::*;

/// 4 physical cores x 2 SMT threads (one package): cpu -> core = cpu/2.
fn layout_4c2t() -> Vec<CoreView> {
    (0..8)
        .map(|cpu| CoreView {
            cpu,
            package: 0,
            core: cpu / 2,
            online: true,
            eligible: cpu != 0,
            awake: false,
            idle: cpu != 0, // worker threads start idle
        })
        .collect()
}

fn set_busy(views: &mut [CoreView], cpu: u32) {
    views[cpu as usize].idle = false;
    views[cpu as usize].eligible = false;
}

#[test]
fn same_core_and_busy_helpers() {
    let mut v = layout_4c2t();
    assert!(same_core(&v[0], &v[1])); // cpu0 and cpu1 share core0
    assert!(!same_core(&v[0], &v[2]));
    assert!(!busy(&v[1])); // idle
    set_busy(&mut v, 0);
    assert!(busy(&v[0]));
    assert!(sibling_busy(&v, 1)); // cpu1's sibling cpu0 is busy
    assert!(!sibling_busy(&v, 2)); // core1 is free
    assert!(!sibling_busy(&v, 0)); // master itself: no *other* busy sibling
}

#[test]
fn prefers_a_fresh_core_over_the_masters_sibling() {
    let mut v = layout_4c2t();
    set_busy(&mut v, 0); // master busy on core0

    // cpu1 shares core0 (master) -> shared; cpu2 is on a fresh core.
    assert!(sibling_busy(&v, 1));
    assert!(!sibling_busy(&v, 2));
    assert_eq!(pick_wake(&v), Some(2));
}

#[test]
fn shares_only_when_every_core_has_a_busy_sibling() {
    let mut v = layout_4c2t();
    // One busy thread on each of the four physical cores.
    for cpu in [0u32, 2, 4, 6] {
        set_busy(&mut v, cpu);
    }
    // Remaining candidates (1,3,5,7) all share a busy core; pick the lowest.
    assert_eq!(pick_wake(&v), Some(1));
}

#[test]
fn master_idle_keeps_its_sibling_fresh() {
    let mut v = layout_4c2t();
    v[0].idle = true; // master is idle, not busy
    assert!(!sibling_busy(&v, 1)); // core0 has no busy thread
    assert_eq!(pick_wake(&v), Some(1)); // lowest fresh candidate
}

#[test]
fn awake_candidates_are_not_woken() {
    let mut v = layout_4c2t();
    for c in v.iter_mut() {
        c.awake = true; // everyone already awake
    }
    assert_eq!(pick_wake(&v), None);
}

#[test]
fn no_candidates_yields_none() {
    let mut v = layout_4c2t();
    for c in v.iter_mut() {
        c.eligible = false;
    }
    assert_eq!(pick_wake(&v), None);
}

#[test]
fn no_smt_every_core_is_fresh() {
    // 4 cores, 1 thread each: cpu -> core = cpu.
    let mut v: Vec<CoreView> = (0..4)
        .map(|cpu| CoreView {
            cpu,
            package: 0,
            core: cpu,
            online: true,
            eligible: cpu != 0,
            awake: false,
            idle: cpu != 0,
        })
        .collect();
    set_busy(&mut v, 0);
    // No siblings exist, so the lowest candidate is chosen.
    assert_eq!(pick_wake(&v), Some(1));
}

#[test]
fn multiple_packages_are_distinct_cores() {
    // Two packages, 2 threads each; same `core` id in each package.
    let v: Vec<CoreView> = (0..4)
        .map(|cpu| CoreView {
            cpu,
            package: cpu / 2,
            core: cpu % 2,
            online: true,
            eligible: cpu != 0,
            awake: false,
            idle: cpu != 0,
        })
        .collect();
    // cpu1 (pkg0/core1) and cpu3 (pkg1/core1) are NOT the same physical core.
    assert!(!same_core(&v[1], &v[3]));
    assert_eq!(pick_wake(&v), Some(1));
}

// C-state ids used by the tests: C1 = 1, C3 = 2.
const C1: u32 = 1;
const C3: u32 = 2;

#[test]
fn deep_idle_is_capped_to_c1_while_the_core_is_shared() {
    let mut v = layout_4c2t();
    set_busy(&mut v, 0); // master busy on core0
    // cpu1 shares core0 -> cannot go deep; cpu2's core1 is free.
    assert_eq!(cap_idle_depth(C3, &v, 1), C1);
    assert_eq!(cap_idle_depth(C3, &v, 2), C3);
    // C1 request is never deepened/changed.
    assert_eq!(cap_idle_depth(C1, &v, 1), C1);
}

#[test]
fn core_offline_requires_the_whole_core_idle() {
    let mut v = layout_4c2t();
    set_busy(&mut v, 0); // master busy on core0
    assert!(!can_offline_core(&v, 1)); // sibling busy -> must not park
    assert!(can_offline_core(&v, 2)); // fresh core -> ok
    // An idle core may be parked (its sibling is idle too).
    let mut v2 = layout_4c2t();
    v2[0].idle = true; // master idle -> core0 fully idle
    assert!(can_offline_core(&v2, 1));
}

#[test]
fn core_mask_groups_smt_siblings() {
    let v = layout_4c2t();
    assert_eq!(core_mask(&v, 0), 0b0000_0011); // core0 = cpu0|cpu1
    assert_eq!(core_mask(&v, 1), 0b0000_0011);
    assert_eq!(core_mask(&v, 2), 0b0000_1100); // core1 = cpu2|cpu3
    assert_eq!(core_mask(&v, 7), 0b1100_0000); // core3 = cpu6|cpu7
    // A CPU index not present in the view set yields an empty mask.
    assert_eq!(core_mask(&v, 99), 0);
}
