//! Unit tests for the pure load-balance policy (`sched/src/balance_policy.rs`).

use cact_host_tests::balance_policy::{pick_migration, CoreLoad};

/// The production threshold (`balance.rs`).
const THRESH: u32 = 150;

fn core(cpu: u32, online: bool, load_permille: u32, runnable: u32) -> CoreLoad {
    CoreLoad {
        cpu,
        online,
        load_permille,
        runnable,
    }
}

#[test]
fn empty_view_is_offline_and_empty() {
    let e = CoreLoad::empty(7);
    assert_eq!(e.cpu, 7);
    assert!(!e.online);
    assert_eq!(e.load_permille, 0);
    assert_eq!(e.runnable, 0);
}

#[test]
fn balanced_machine_migrates_nothing() {
    let v = vec![
        core(0, true, 800, 1),
        core(1, true, 750, 1),
        core(2, true, 700, 0),
    ];
    // Spread is 100 per-mille, below the 150 threshold.
    assert_eq!(pick_migration(&v, THRESH), None);
}

#[test]
fn busiest_with_work_moves_to_the_least_loaded() {
    let v = vec![
        core(0, true, 900, 0),  // master busy, but nothing queued behind it
        core(1, true, 950, 4),  // busiest core that holds runnable work
        core(2, true, 300, 0),
        core(3, true, 100, 0),  // least loaded
    ];
    assert_eq!(pick_migration(&v, THRESH), Some((1, 3)));
}

#[test]
fn a_core_busy_only_with_its_running_task_is_not_a_source() {
    let v = vec![
        core(0, true, 1000, 0), // busiest overall, but it can give nothing away
        core(1, true, 400, 1),
        core(2, true, 100, 0),
    ];
    // src skips cpu0 (runnable == 0) and picks cpu1; dst is cpu2.
    assert_eq!(pick_migration(&v, THRESH), Some((1, 2)));
}

#[test]
fn offline_cores_are_neither_sources_nor_targets() {
    let v = vec![
        core(0, true, 500, 2),
        core(1, false, 0, 0), // parked: not a source, and not a free target either
        core(2, true, 200, 0),
    ];
    assert_eq!(pick_migration(&v, THRESH), Some((0, 2)));
}

#[test]
fn no_online_peer_yields_none() {
    let v = vec![core(0, true, 900, 3)];
    assert_eq!(pick_migration(&v, THRESH), None);
}

#[test]
fn no_runnable_work_anywhere_yields_none() {
    let v = vec![core(0, true, 1000, 0), core(1, true, 100, 0)];
    assert_eq!(pick_migration(&v, THRESH), None);
}

#[test]
fn spread_exactly_at_the_threshold_migrates() {
    let v = vec![core(0, true, 650, 1), core(1, true, 500, 0)];
    assert_eq!(pick_migration(&v, THRESH), Some((0, 1)));
}

#[test]
fn ties_break_to_the_lowest_cpu_index() {
    let v = vec![
        core(0, true, 900, 1),
        core(1, true, 900, 1),
        core(2, true, 200, 0),
        core(3, true, 200, 0),
    ];
    // Busiest tie -> cpu0; least-loaded tie -> cpu2.
    assert_eq!(pick_migration(&v, THRESH), Some((0, 2)));
}
