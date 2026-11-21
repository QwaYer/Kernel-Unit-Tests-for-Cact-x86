# Kernel-Unit-Tests-for-Cact

Host-side unit tests for pure CactOS kernel logic (PLAN-2.0.0 **P2.1**). These
run on the build machine with `cargo test` — no QEMU, no hardware — and cover
logic that has **no hardware access, no globals and no FFI**. Logic that touches
a device or a kernel global is no longer covered by kernel boot self-tests (they
were removed); it is exercised by the headless boot check
(`CactKernel-x86_64/tools/cact_check.sh`) and, on hardware, by hand.

The modules under test are `#[path]`-included from the real kernel source (see
`src/lib.rs`), so a regression here is a regression in the kernel, and the tests
cannot drift from copied code.

## Run

```
cargo test
```

## Currently covered

**120 tests** across fourteen modules:

| Module | Kernel source | What it checks |
| --- | --- | --- |
| `energy_model` | `Cact/kernel/proc/sched/src/energy_model.rs` | C-state ids/energy costs, the benefit/cost wake formula, load/trend thresholds (14) |
| `syncobj_state` | `Cact/drivers/gpu/drm/rust_drm/src/syncobj_state.rs` | the fence-free syncobj state: binary vs timeline reachability, point monotonicity, which points an object takes, `TRANSFER` shapes (14) |
| `placement` | `Cact/kernel/proc/sched/src/placement.rs` | sibling-aware core placement, C-state depth cap, core offlining (11) |
| `balance_policy` | `Cact/kernel/proc/sched/src/balance_policy.rs` | load-balance core selection: busiest-with-work → least-loaded, thresholds, offline/empty handling, tie-breaks (9) |
| `topology` | `Cact/kernel/proc/sched/src/topology.rs` | CPUID leaf `0xB`/`0x1F` decode plus the HTT fallback (9) |
| `intrusive_queue` | `Cact/kernel/proc/sched/src/intrusive_queue.rs` | the intrusive FIFO the MLFQ drives, including a non-unlinking `head()` peek (8) |
| `mlfq_policy` | `Cact/kernel/proc/sched/src/mlfq_policy.rs` | MLFQ levels/quanta/boost/clamp/pick (6) |
| `pmm_math` | `Cact/kernel/memory/rust_mm/src/pmm_math.rs` | PMM bitmap index/mask, page↔addr, region clip, `ram_end`/MMIO window (6) |
| `munmap_plan` | `Cact/kernel/memory/rust_mm/src/munmap_plan.rs` | munmap range clipping: whole/front/back/middle split, adjacency (exclusive ends), no wrap (8) |
| `realloc_plan` | `Cact/kernel/memory/rust_mm/src/alloc/realloc_plan.rs` | `krealloc` sizing: required block size (plain vs aligned offset), in-place vs absorb-next-free vs move, split threshold, saturation (12) |
| `heap_stats` | `Cact/kernel/memory/rust_mm/src/alloc/heap_stats.rs` | heap-walk aggregation: free/used split, free-block count, largest free block, saturating totals (6) |
| `heap_class` | `Cact/kernel/memory/rust_mm/src/alloc/heap_class.rs` | segregated free-list size classes: `ceil_log2` mapping, power-of-two class sizes, monotonicity, top-class clamp (6) |
| `slab_owner` | `Cact/kernel/memory/rust_mm/src/alloc/slab_owner.rs` | slab-page ownership: magic wins over the old capacity guess, a heap page is never a slab, zero capacity / null cache rejected (6) |
| `vfs_pure` | `Cact/fs/rust_vfs/src/vfs_pure.rs` | dcache bucket index and the child→parent table (5) |

## Adding a target

1. Move the pure logic into a `no_std`, dependency-free module in the kernel
   crate (no `crate::` references, no FFI) and have the kernel call it, so both
   the kernel and the tests share one source of truth.
2. Add a `#[path = "..."] pub mod <name>;` line to `src/lib.rs`.
3. Add `tests/<name>.rs`.
