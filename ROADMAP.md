# underust roadmap

Design spec lives in the private vault; this file is the public working view.

**Shipped:** M0 (the harness) and M0.5 (the vertical slice). **5 tasks, 5 modes, 5 of 22
tracks with content.** Everything past M0.5 is gated -- see the note below.

## Milestones

| | Milestone | Content | Status |
|---|---|---|---|
| M0 | Foundation | workspace, three crates, CLI, seven CI gates, Dockerfile, release automation | **shipped** |
| M0.5 | Vertical slice | one exemplar per instrument-graded mode (5 tasks) | **shipped** |
| M1 | Stable core | depth in `drop`, `alloc`, `weak`, `layout` | gated |
| M2 | Compile tracks | `own`, `borrowck`, `lifetime` + CONSTRAIN | gated |
| M3 | Nightly tracks | `unsafe` + SOUNDNESS, `mono`, `macro` | gated |
| M4 | Concurrency | `atomics`, `conc`, `async` | gated |
| M5 | The rest | `codegen`, `iter`, `edges`, `const`, `pin`, `traits` | gated |
| M6 | Judgement lanes | `review`, `design` | gated |
| M7 | Radar sweep | releases 1.85 onward, `versions` track | partly open |

## Candidate pool

Rows are marked when built, never deleted.

| Release | Note | Track | Built |
|---|---|---|---|
| 1.98 | `&mut` lifetime shortening under unsize coercion, even in invariant position | `lifetime` | |
| 1.98 | stricter `repr(transparent)` validation | `layout` | |
| 1.98 | `transmute()` size checking corrected | `layout` | |
| 1.98 | runtime-symbol lints covering `memcmp` and `strlen` | `unsafe` | |
| 1.98 | floating-point algebraic operations stabilised | `edges` | |
| — | Polonius Alpha: flow-sensitive borrow checking, `-Zpolonius=off` reverts to NLL | `borrowck` | |

## Invalidated by a release

Empty. The first entry is expected to be a `borrowck` task, when Polonius stabilises — see
`radar/versions/polonius.md`.
