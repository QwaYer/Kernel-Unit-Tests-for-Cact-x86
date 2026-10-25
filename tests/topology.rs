//! Unit tests for the pure CPUID topology decoder (`sched/src/topology.rs`).

use cact_host_tests::topology::*;

#[test]
fn decode_level_masks_register_fields() {
    let l = decode_level(0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF);
    assert_eq!(l.shift, 0x1F);
    assert_eq!(l.count, 0xFFFF);
    assert_eq!(l.kind, 0xFF);

    // Level *type* is ECX[15:8]; the low byte is the (ignored) level number.
    let l2 = decode_level(3, 8, (LEVEL_CORE << 8) | 1);
    assert_eq!(l2, Level { shift: 3, count: 8, kind: LEVEL_CORE });
    let l3 = decode_level(1, 2, (LEVEL_SMT << 8) | 0);
    assert_eq!(l3.kind, LEVEL_SMT);
    // A type of 0 (ECX[15:8] == 0) is invalid regardless of the level number.
    assert_eq!(decode_level(0, 1, 7).kind, LEVEL_INVALID);
}

#[test]
fn four_cores_two_threads() {
    let smt = Level { shift: 1, count: 2, kind: LEVEL_SMT };
    let core = Level { shift: 3, count: 8, kind: LEVEL_CORE };
    let t = topology_from_levels(Some(smt), Some(core)).unwrap();

    assert_eq!(t.threads_per_core, 2);
    assert_eq!(t.cores_per_package, 4);
    assert_eq!(t.logical_per_package, 8);
    assert_eq!(t.smt_shift, 1);
    assert_eq!(t.core_shift, 3);
}

#[test]
fn package_and_core_ids_across_sockets() {
    let smt = Level { shift: 1, count: 2, kind: LEVEL_SMT };
    let core = Level { shift: 3, count: 8, kind: LEVEL_CORE };
    let t = topology_from_levels(Some(smt), Some(core)).unwrap();

    assert_eq!(package_id(&t, 0), 0);
    assert_eq!(core_id(&t, 0), 0);
    assert_eq!(core_id(&t, 1), 0); // second thread of core 0
    assert_eq!(core_id(&t, 6), 3); // (6>>1)%4
    assert_eq!(package_id(&t, 6), 0);
    assert_eq!(package_id(&t, 8), 1); // first logical of socket 1
    assert_eq!(core_id(&t, 8), 0);
    assert_eq!(core_id(&t, 10), 1);
}

#[test]
fn single_thread_without_smt_level() {
    let core = Level { shift: 2, count: 4, kind: LEVEL_CORE };
    let t = topology_from_levels(None, Some(core)).unwrap();
    assert_eq!(t.threads_per_core, 1);
    assert_eq!(t.cores_per_package, 4);
    assert_eq!(t.logical_per_package, 4);
    assert_eq!(t.smt_shift, 0);
    assert_eq!(t.core_shift, 2);
    assert_eq!(package_id(&t, 5), 1);
    assert_eq!(core_id(&t, 5), 1);
}

#[test]
fn no_levels_is_a_single_logical_cpu() {
    let t = topology_from_levels(None, None).unwrap();
    assert_eq!(t.threads_per_core, 1);
    assert_eq!(t.cores_per_package, 1);
    assert_eq!(t.logical_per_package, 1);
    assert_eq!(package_id(&t, 7), 7);
    assert_eq!(core_id(&t, 7), 0);
}

#[test]
fn contradictory_levels_are_rejected() {
    // 4 threads per core but only 2 logical processors per package.
    let smt = Level { shift: 2, count: 4, kind: LEVEL_SMT };
    let core = Level { shift: 3, count: 2, kind: LEVEL_CORE };
    assert_eq!(topology_from_levels(Some(smt), Some(core)), None);
}

#[test]
fn wrong_kind_is_ignored() {
    // A level tagged CORE in the SMT slot must be ignored (defaults to 1 thread).
    let smt = Level { shift: 1, count: 2, kind: LEVEL_CORE };
    let t = topology_from_levels(Some(smt), None).unwrap();
    assert_eq!(t.threads_per_core, 1);
}

#[test]
fn htt_fallback_helpers() {
    assert_eq!(logical_per_package_from_leaf1(0x0008_0000), 8);
    assert_eq!(logical_per_package_from_leaf1(0x0001_0000), 1);
    assert_eq!(cores_per_package_from_leaf4(0x0400_0000), 2);
    assert_eq!(cores_per_package_from_leaf4(0x0000_0000), 1);
}

#[test]
fn htt_fallback_topology() {
    let t = topology_from_fallback(8, 4).unwrap();
    assert_eq!(t.threads_per_core, 2);
    assert_eq!(t.cores_per_package, 4);
    assert_eq!(t.smt_shift, 1);
    assert_eq!(t.core_shift, 3); // package shift = log2(logical per package)
    assert_eq!(package_id(&t, 9), 1);
    assert_eq!(core_id(&t, 9), 0);

    assert_eq!(topology_from_fallback(0, 4), None);
    assert_eq!(topology_from_fallback(2, 4), None); // logical < cores
}
