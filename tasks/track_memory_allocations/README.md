# Oxc allocations stats

This task keeps track of the number of system allocations as well as arena allocations and the total number of bytes allocated. This is used for monitoring possible regressions in allocations and improvements in memory usage.

Update the snapshots with `cargo allocs`.

## Metrics

Recorded per file and per stage (parser, semantic, transformer, minifier, formatter):

- `file size` - size of the source text
- `sys allocs` / `sys reallocs` / `sys deallocs` - system allocator activity during the stage
- `sys alloc bytes` - total bytes requested from the system allocator during the stage (allocation sizes plus reallocation growth)
- `sys peak growth` - high-water mark of live heap memory during the stage, relative to the live bytes when the stage started; captures transient heap the stage needs even when it frees it again before finishing
- `arena allocs` / `arena reallocs` - arena allocator activity during the stage; semantic analysis reports its scoping-owned arena, while other stages report the outer parser/task arena
- `arena size` - bytes used in the outer parser/task arena at the end of the stage (cumulative: the arena is not reset between stages, so this includes the AST built by the parser); this does not include scoping's internal arena

Arena chunk allocations and deallocations are excluded from all `sys` metrics: chunk sizes are quantized and platform-dependent (whether an arena needs one more chunk depends on byte totals that vary across architectures), and for long-lived arenas chunk growth reflects the arena's history rather than the stage's own behavior. Arenas mark their chunk operations for the tracking allocator to skip.

## Cross-platform reproducibility

Allocation counts are snapshotted exactly: the task pins mimalloc as the global allocator and excludes arena chunk allocations from the system allocation counts. This removes chunk-growth variation, but does not guarantee that HashMap growth or rebuilding performs the same number of allocations on every target. Changes that expose previously untracked allocations should be checked on the relevant local and CI targets before accepting a shared snapshot; the byte tolerances below do not apply to counts.

`arena size` is not exactly reproducible: type layout differs slightly across architectures (e.g. hashbrown's table control groups are 16 bytes on x86_64 vs 8 bytes on aarch64), which shifts arena byte totals by up to ~128 bytes per file. So that a snapshot generated on any platform passes CI on the others, the snapshots are YAML: `cargo allocs` parses the committed snapshot and keeps a byte metric's committed value when the measured one is within that metric's tolerance (`arena size`: 1024 bytes; `sys alloc bytes` and `sys peak growth`: 1%, minimum 4 kB); only differences beyond the tolerance rewrite the value.
