//! Unit tests for the pure intrusive FIFO (`sched/src/intrusive_queue.rs`).

use cact_host_tests::intrusive_queue::{IntrusiveQueue, Link};
use std::ptr;

#[repr(C)]
struct Node {
    val: u32,
    next: *mut Node,
}

impl Link for Node {
    fn next(&self) -> *mut Node {
        self.next
    }
    fn set_next(&mut self, next: *mut Node) {
        self.next = next;
    }
}

fn node(v: u32) -> Node {
    Node { val: v, next: ptr::null_mut() }
}

fn val(p: *mut Node) -> u32 {
    assert!(!p.is_null(), "popped a null node");
    // SAFETY: the pointer came from a live node pushed by the test.
    unsafe { (*p).val }
}

#[test]
fn empty_queue_reports_empty() {
    let mut q: IntrusiveQueue<Node> = IntrusiveQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.count(), 0);
    assert!(q.pop().is_null());
}

#[test]
fn fifo_order_and_count() {
    let mut q = IntrusiveQueue::new();
    let (mut a, mut b, mut c) = (node(1), node(2), node(3));

    q.push(&mut a);
    q.push(&mut b);
    q.push(&mut c);
    assert_eq!(q.count(), 3);
    assert!(!q.is_empty());

    assert_eq!(val(q.pop()), 1);
    assert_eq!(q.count(), 2);
    assert_eq!(val(q.pop()), 2);
    assert_eq!(val(q.pop()), 3);
    assert!(q.pop().is_null());
    assert!(q.is_empty());
    assert_eq!(q.count(), 0);
}

#[test]
fn popped_node_link_is_cleared() {
    let mut q = IntrusiveQueue::new();
    let mut a = node(1);
    let mut b = node(2);
    q.push(&mut a);
    q.push(&mut b);
    let _ = q.pop();
    assert!(a.next.is_null());
}

#[test]
fn remove_head_middle_tail() {
    // Middle
    let mut q = IntrusiveQueue::new();
    let (mut a, mut b, mut c) = (node(1), node(2), node(3));
    q.push(&mut a);
    q.push(&mut b);
    q.push(&mut c);
    assert!(q.remove(&mut b as *mut Node));
    assert_eq!(q.count(), 2);
    assert_eq!(val(q.pop()), 1);
    assert_eq!(val(q.pop()), 3);

    // Head
    let mut q = IntrusiveQueue::new();
    let (mut a, mut b) = (node(10), node(20));
    q.push(&mut a);
    q.push(&mut b);
    assert!(q.remove(&mut a as *mut Node));
    assert_eq!(q.count(), 1);
    assert_eq!(val(q.pop()), 20);

    // Tail
    let mut q = IntrusiveQueue::new();
    let (mut a, mut b) = (node(10), node(20));
    q.push(&mut a);
    q.push(&mut b);
    assert!(q.remove(&mut b as *mut Node));
    assert_eq!(q.count(), 1);
    assert_eq!(val(q.pop()), 10);
    assert!(q.is_empty());
}

#[test]
fn remove_absent_and_null_are_noops() {
    let mut q = IntrusiveQueue::new();
    let mut a = node(1);
    let mut outsider = node(99);
    q.push(&mut a);

    assert!(!q.remove(&mut outsider as *mut Node));
    assert_eq!(q.count(), 1);
    assert!(!q.remove(ptr::null_mut()));
    assert_eq!(q.count(), 1);
    assert_eq!(val(q.pop()), 1);
}

#[test]
fn node_can_be_reused_after_pop() {
    let mut q = IntrusiveQueue::new();
    let mut a = node(1);
    q.push(&mut a);
    let _ = q.pop();

    let mut b = node(2);
    q.push(&mut b);
    q.push(&mut a); // re-push the previously popped node
    assert_eq!(q.count(), 2);
    assert_eq!(val(q.pop()), 2);
    assert_eq!(val(q.pop()), 1);
}

#[test]
fn head_peeks_without_unlinking() {
    let mut q: IntrusiveQueue<Node> = IntrusiveQueue::new();
    assert!(q.head().is_null());

    let (mut a, mut b) = (node(1), node(2));
    q.push(&mut a);
    q.push(&mut b);

    assert_eq!(val(q.head()), 1);
    assert_eq!(q.count(), 2); // peeking leaves the queue untouched
    assert_eq!(val(q.pop()), 1);
    assert_eq!(val(q.head()), 2);
    let _ = q.pop();
    assert!(q.head().is_null());
}

#[test]
fn clear_drops_everything() {
    let mut q = IntrusiveQueue::new();
    let (mut a, mut b) = (node(1), node(2));
    q.push(&mut a);
    q.push(&mut b);
    q.clear();
    assert!(q.is_empty());
    assert_eq!(q.count(), 0);
    // The nodes' links are cleared so they can be reused.
    assert!(a.next.is_null());
    assert!(b.next.is_null());
}
