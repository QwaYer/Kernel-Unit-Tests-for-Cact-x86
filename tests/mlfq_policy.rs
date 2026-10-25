//! Unit tests for the pure MLFQ policy (`sched/src/mlfq_policy.rs`).

use cact_host_tests::mlfq_policy::*;

#[test]
fn constants_are_pinned() {
    assert_eq!(MLFQ_LEVELS, 4);
    assert_eq!(MLFQ_LEVEL_RT, 0);
    assert_eq!(MLFQ_LEVEL_INTERACTIVE, 1);
    assert_eq!(MLFQ_LEVEL_NORMAL, 2);
    assert_eq!(MLFQ_LEVEL_BACKGROUND, 3);
    assert_eq!(MLFQ_QUANTUM, [5, 1, 2, 4]);
    assert_eq!(BOOST_INTERVAL, 50);
    assert_eq!(BOOST_TARGET, MLFQ_LEVEL_INTERACTIVE);
}

#[test]
fn clamp_level_bounds_to_range() {
    assert_eq!(clamp_level(0), 0);
    assert_eq!(clamp_level(1), 1);
    assert_eq!(clamp_level(2), 2);
    assert_eq!(clamp_level(3), 3);
    assert_eq!(clamp_level(4), 3);
    assert_eq!(clamp_level(99), 3);
}

#[test]
fn quantum_for_level() {
    assert_eq!(quantum_for(0), 5);
    assert_eq!(quantum_for(1), 1);
    assert_eq!(quantum_for(2), 2);
    assert_eq!(quantum_for(3), 4);
    // Out-of-range clamps to the background quantum.
    assert_eq!(quantum_for(9), 4);
}

#[test]
fn pick_highest_level_selects_lowest_index() {
    assert_eq!(pick_highest_level(&[0, 0, 0, 0]), None);
    assert_eq!(pick_highest_level(&[1, 0, 0, 0]), Some(0));
    assert_eq!(pick_highest_level(&[0, 1, 0, 0]), Some(1));
    assert_eq!(pick_highest_level(&[0, 0, 2, 0]), Some(2));
    assert_eq!(pick_highest_level(&[0, 0, 0, 5]), Some(3));
    // Highest priority (lowest index) wins even when lower ones are busier.
    assert_eq!(pick_highest_level(&[1, 9, 9, 9]), Some(0));
}

#[test]
fn demote_on_quantum_skips_rt_and_floors_at_background() {
    assert_eq!(demote_on_quantum(MLFQ_LEVEL_RT), MLFQ_LEVEL_RT); // RT never drops
    assert_eq!(demote_on_quantum(1), 2);
    assert_eq!(demote_on_quantum(2), 3);
    assert_eq!(demote_on_quantum(MLFQ_LEVEL_BACKGROUND), MLFQ_LEVEL_BACKGROUND); // floor
}

#[test]
fn boost_due_at_interval() {
    assert!(!boost_due(0));
    assert!(!boost_due(49));
    assert!(boost_due(50));
    assert!(boost_due(51));
    assert!(boost_due(1000));
}
