//! Host harness for pure CactOS kernel logic (PLAN-2.0.0 P2.1).
//!
//! Each module below is `#[path]`-included from the **actual kernel source**,
//! not copied, so the tests can never drift from what the kernel compiles.
//! Only logic with no hardware access, no globals and no FFI belongs here;
//! anything that touches a device or a kernel global is exercised by the
//! headless boot check (`CactKernel-x86_64/tools/cact_check.sh`).
//!
//! Run with:  cargo test   (from this directory)

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/energy_model.rs"]
pub mod energy_model;

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/topology.rs"]
pub mod topology;

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/placement.rs"]
pub mod placement;

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/balance_policy.rs"]
pub mod balance_policy;

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/intrusive_queue.rs"]
pub mod intrusive_queue;

#[path = "../../CactKernel-x86_64/Cact/kernel/proc/sched/src/mlfq_policy.rs"]
pub mod mlfq_policy;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/pmm_math.rs"]
pub mod pmm_math;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/munmap_plan.rs"]
pub mod munmap_plan;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/alloc/realloc_plan.rs"]
pub mod realloc_plan;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/alloc/heap_stats.rs"]
pub mod heap_stats;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/alloc/heap_class.rs"]
pub mod heap_class;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/alloc/slab_owner.rs"]
pub mod slab_owner;

#[path = "../../CactKernel-x86_64/Cact/kernel/memory/rust_mm/src/comp/zpool_alloc.rs"]
pub mod zpool_alloc;

#[path = "../../CactKernel-x86_64/Cact/fs/rust_vfs/src/vfs_pure.rs"]
pub mod vfs_pure;

#[path = "../../CactKernel-x86_64/Cact/drivers/gpu/drm/rust_drm/src/syncobj_state.rs"]
pub mod syncobj_state;
