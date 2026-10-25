//! Unit tests for the fence-free syncobj state (`rust_drm/src/syncobj_state.rs`).
//!
//! The module under test is the real kernel source, so a regression here is a
//! regression in what `DRM_IOCTL_SYNCOBJ_{WAIT,TRANSFER,EVENTFD}` decide — the
//! part of the syncobj support that a boot check cannot reach, because it needs
//! a display driver and a userspace DRM client.

use cact_host_tests::syncobj_state::{SyncState, TransferError};

fn binary(signaled: bool) -> SyncState {
    SyncState {
        timeline: false,
        signaled,
        point: 0,
    }
}

fn timeline_at(point: u64) -> SyncState {
    SyncState {
        timeline: true,
        signaled: false,
        point,
    }
}

/* ── the binary flavour ─────────────────────────────────────────────────── */

#[test]
fn binary_is_not_reached_until_signalled() {
    let mut st = binary(false);
    assert!(!st.reached(0));
    st.signal();
    assert!(st.reached(0));
}

#[test]
fn binary_ignores_points() {
    // A binary object has one bit; asking for a point is asking for that bit.
    let st = binary(true);
    assert!(st.reached(0));
    assert!(st.reached(1));
    assert!(st.reached(u64::MAX));
}

#[test]
fn binary_reset_clears_it() {
    let mut st = binary(true);
    st.reset();
    assert!(!st.reached(0));
}

#[test]
fn binary_grows_no_point() {
    // The bit is the whole state: signalling must not invent a point, or a later
    // transfer of a point would see one that never existed.
    let mut st = binary(false);
    st.signal();
    assert_eq!(st.point, 0);
}

/* ── the timeline flavour ───────────────────────────────────────────────── */

#[test]
fn fresh_timeline_has_reached_nothing() {
    let st = timeline_at(0);
    assert!(!st.reached(0)); // not even "any point at all"
    assert!(!st.reached(1));
}

#[test]
fn timeline_reaches_its_point_and_no_further() {
    let st = timeline_at(3);
    assert!(st.reached(0)); // "any point at all"
    assert!(st.reached(3));
    assert!(!st.reached(4));
}

#[test]
fn point_only_moves_forward() {
    let mut st = timeline_at(3);

    assert!(st.advance(5));
    assert_eq!(st.point, 5);

    // A signal at the point it already has, or below it, changes nothing — which
    // is what tells a signaler that no registered eventfd can have become ready.
    assert!(!st.advance(5));
    assert!(!st.advance(4));
    assert_eq!(st.point, 5);
}

#[test]
fn timeline_signal_bit_is_meaningless() {
    // `signal()` exists for the binary flavour; a timeline object is told by its
    // point, so a stray bit must not make it look reached.
    let mut st = timeline_at(0);
    st.signal();
    assert!(!st.reached(0));
    assert!(!st.reached(1));
}

/* ── which points an object can be asked about ──────────────────────────── */

#[test]
fn binary_takes_no_point() {
    let st = binary(false);
    assert!(st.takes_point(0));
    assert!(!st.takes_point(1));
    assert!(!st.takes_point(u64::MAX));
}

#[test]
fn timeline_takes_any_point() {
    let st = timeline_at(0);
    assert!(st.takes_point(0));
    assert!(st.takes_point(1));
    assert!(st.takes_point(u64::MAX));
}

/* ── transfer ───────────────────────────────────────────────────────────── */

#[test]
fn transfer_on_a_binary_destination_signals_it() {
    let mut st = binary(false);
    assert_eq!(st.take_transfer(0), Ok(true));
    assert!(st.reached(0));

    // A second transfer of the same state is legal and changes nothing.
    assert_eq!(st.take_transfer(0), Ok(false));
    assert!(st.reached(0));
}

#[test]
fn transfer_on_a_timeline_destination_moves_the_point() {
    let mut st = timeline_at(1);
    assert_eq!(st.take_transfer(5), Ok(true));
    assert_eq!(st.point, 5);

    // A point the destination has already passed changes nothing...
    assert_eq!(st.take_transfer(3), Ok(false));
    assert_eq!(st.point, 5);

    // ...and a point that is not a point of this object is still refused.
    assert_eq!(st.take_transfer(0), Err(TransferError::WrongShape));
}

#[test]
fn transfer_refuses_the_other_flavour() {
    // A point belongs to a timeline object...
    assert_eq!(binary(false).take_transfer(1), Err(TransferError::WrongShape));
    // ...and the signalled bit to a binary one.
    assert_eq!(
        timeline_at(0).take_transfer(0),
        Err(TransferError::WrongShape)
    );
}

#[test]
fn refused_transfer_leaves_the_destination_alone() {
    let mut st = binary(true);
    assert!(st.take_transfer(9).is_err());
    assert!(st.signaled);

    let mut st = timeline_at(2);
    assert!(st.take_transfer(0).is_err());
    assert_eq!(st.point, 2);
}
