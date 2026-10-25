//! Unit tests for the pure energy-governor model (`sched/src/energy_model.rs`).
//!
//! The module under test is the real kernel source, so a regression here is a
//! regression in the kernel's benefit/cost decision.

use cact_host_tests::energy_model::*;

#[test]
fn cstate_ids_are_ordered_and_distinct() {
    assert_eq!(CSTATE_C0, 0);
    assert_eq!(CSTATE_C1, 1);
    assert_eq!(CSTATE_C3, 2);
    assert_eq!(CSTATE_C6, 3);
    assert!(CSTATE_C0 < CSTATE_C1 && CSTATE_C1 < CSTATE_C3 && CSTATE_C3 < CSTATE_C6);
}

#[test]
fn trend_constants_match_energy_h() {
    assert_eq!(TREND_DOWN, -1);
    assert_eq!(TREND_STABLE, 0);
    assert_eq!(TREND_UP, 1);
}

#[test]
fn energy_costs_are_pinned_and_monotonic_with_depth() {
    // Exact published values; pinning them makes an accidental table edit fail
    // here instead of silently shifting the governor's behaviour.
    assert_eq!(wakeup_energy(CSTATE_C0), 0);
    assert_eq!(wakeup_energy(CSTATE_C1), 5);
    assert_eq!(wakeup_energy(CSTATE_C3), 40);
    assert_eq!(wakeup_energy(CSTATE_C6), 160);

    assert_eq!(cache_harm_energy(CSTATE_C0), 0);
    assert_eq!(cache_harm_energy(CSTATE_C1), 1);
    assert_eq!(cache_harm_energy(CSTATE_C3), 30);
    assert_eq!(cache_harm_energy(CSTATE_C6), 100);

    // Deeper states never get cheaper.
    assert!(wakeup_energy(CSTATE_C0) <= wakeup_energy(CSTATE_C1));
    assert!(wakeup_energy(CSTATE_C1) <= wakeup_energy(CSTATE_C3));
    assert!(wakeup_energy(CSTATE_C3) <= wakeup_energy(CSTATE_C6));
    assert!(cache_harm_energy(CSTATE_C0) <= cache_harm_energy(CSTATE_C1));
    assert!(cache_harm_energy(CSTATE_C1) <= cache_harm_energy(CSTATE_C3));
    assert!(cache_harm_energy(CSTATE_C3) <= cache_harm_energy(CSTATE_C6));
}

#[test]
fn cost_is_wakeup_plus_cache_harm() {
    assert_eq!(cost(CSTATE_C0), 0);
    assert_eq!(cost(CSTATE_C1), 6);
    assert_eq!(cost(CSTATE_C3), 70);
    assert_eq!(cost(CSTATE_C6), 260);
}

#[test]
fn cost_out_of_range_state_is_zero() {
    assert_eq!(wakeup_energy(99), 0);
    assert_eq!(cache_harm_energy(99), 0);
    assert_eq!(cost(99), 0);
}

#[test]
fn work_cycles_scales_with_queue_and_ipl() {
    assert_eq!(work_cycles(0, 1), 0);
    assert_eq!(work_cycles(1, 1), 1_000_000);
    assert_eq!(work_cycles(3, 1), 3_000_000);
    assert_eq!(work_cycles(2, 4), 8_000_000);
    assert!(work_cycles(1, 2) > work_cycles(1, 1));
}

#[test]
fn ipl_zero_is_treated_as_one() {
    assert_eq!(work_cycles(2, 0), work_cycles(2, 1));
}

#[test]
fn benefit_equals_queue_times_ipl() {
    // With the chosen constants (AVG_TASK_CYCLES = 1e6, FREQ/PPC = 1) the
    // benefit collapses to queue_len * ipl -- a stable, checkable identity.
    assert_eq!(benefit(0, 1), 0);
    assert_eq!(benefit(1, 1), 1);
    assert_eq!(benefit(7, 1), 7);
    assert_eq!(benefit(2, 5), 10);
}

#[test]
fn benefit_is_monotonic_and_saturates_instead_of_panicking() {
    assert!(benefit(10, 1) > benefit(9, 1));
    // Must not overflow/panic on extreme inputs (saturating arithmetic).
    let _ = benefit(u32::MAX, u32::MAX);
    let _ = work_cycles(u32::MAX, u32::MAX);
}

#[test]
fn c0_core_is_never_woken() {
    assert_eq!(should_wake(CSTATE_C0, 1000, 1, 900, TREND_UP), 0);
    assert_eq!(should_wake(CSTATE_C0, 0, 1, 900, TREND_UP), 0);
}

#[test]
fn deep_core_wakes_on_large_queue() {
    // benefit(1000,1)=1000 > cost(C6)*1.5=390
    assert_eq!(should_wake(CSTATE_C6, 1000, 1, 0, TREND_DOWN), 1);
    // C3 cost is 70, threshold 105: 100 does not pay, 700 does.
    assert_eq!(should_wake(CSTATE_C3, 100, 1, 0, TREND_STABLE), 0);
    assert_eq!(should_wake(CSTATE_C3, 700, 1, 0, TREND_STABLE), 1);
}

#[test]
fn light_load_does_not_wake() {
    assert_eq!(should_wake(CSTATE_C6, 1, 1, 100, TREND_STABLE), 0);
}

#[test]
fn overload_wakes_ignoring_trend() {
    assert_eq!(should_wake(CSTATE_C1, 0, 1, 900, TREND_STABLE), 1);
    assert_eq!(should_wake(CSTATE_C1, 0, 1, 900, TREND_UP), 1);
    // Exactly 700 permille is not "> 70%".
    assert_eq!(should_wake(CSTATE_C6, 0, 1, 700, TREND_STABLE), 0);
    assert_eq!(should_wake(CSTATE_C6, 0, 1, 701, TREND_STABLE), 1);
}

#[test]
fn sleep_policy_thresholds() {
    assert_eq!(should_sleep(100, 150), 1);
    assert_eq!(should_sleep(100, 50), 0);
    assert_eq!(should_sleep(900, 150), 0);
    // Boundary: load must be strictly below 40% and idle at least 100 ms.
    assert_eq!(should_sleep(400, 100), 0);
    assert_eq!(should_sleep(399, 100), 1);
    assert_eq!(should_sleep(399, 99), 0);
}
