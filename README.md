# underust

A **Rust internals gym**: small, self-contained tasks whose answers are *computed by an
instrument* rather than asserted by an author. Fork it, solve tasks in a gitignored
workspace, and the harness tells you whether reality agrees with you.

Sibling of [undergo](https://github.com/RomanAgaltsev/undergo), which does the same for Go.
underust inherits its mechanics and none of its code.

## The mastery bar

**Predict the behaviour, then cite the mechanism.**

A task is not passed by producing output that happens to be correct. It is passed by having
said, in advance, what the machine would do — and being right for the stated reason. Every
task's README ends with what you should be able to explain, and the harness cannot check
that part. You can.

## The organising rule

> A track ships only when an instrument computes its answers.

A drop log, a counting allocator, `Rc::strong_count`, `miri`, `loom` and rustc's own
diagnostics are not opinions. Where no instrument can settle a question, the task does not
ship. This is what keeps the answers honest — and what makes them checkable by you rather
than taken on trust.

## Getting started

```sh
rustup toolchain install 1.98.1
cargo run -p underust -- doctor
```

`doctor` reports what this machine can grade and prints the exact command that fixes each
gap. It never fails: a machine with nothing installed is precisely the one that needs the
output.

Then:

```sh
cargo run -p underust -- list
cargo run -p underust -- test drop/01-field-order
```

You solve in `work/<task-id>/`, which is gitignored. The task directories themselves stay
pristine — your files are laid over them for the duration of a run and removed afterwards.

## What ships today

**5 tasks, 5 modes, 5 of 22 catalogued tracks with content.**

| Mode | Task | Graded by |
|---|---|---|
| PREDICT | `drop/01-field-order` | a drop log — and it **seals no answer at all** |
| BUILD | `weak/01-break-the-cycle` | `Rc::strong_count`, and a `Weak` that must fail to upgrade |
| OPTIMIZE | `alloc/01-allocation-count` | a counting global allocator — **allocations, never time** |
| CONSTRAIN | `own/01-no-clone` | the compiler, plus a ban checked against the parsed AST |
| SOUNDNESS | `unsafe/01-aliasing` | **miri** — the author asserts nothing |

REVIEW and DESIGN exist in the harness but ship no content yet: they are graded by human
judgement against a rubric, which is the one thing an instrument cannot supply.

## Modes

- **BUILD** — implement it; the tests decide.
- **PREDICT** — commit a guess *before* measuring. Stores no answer: the program computes
  reality at run time and the harness diffs it against what you said.
- **OPTIMIZE** — make it cheaper under a correctness floor. Graded on deterministic costs
  such as allocation counts, never on wall-clock time, which would assert the properties of
  your machine rather than of your code.
- **CONSTRAIN** — make it compile *without the escape hatch*. The ban is declared as data
  and checked on the syntax tree, so a comment mentioning `clone` does not trip it.
- **SOUNDNESS** — the code passes its tests and is unsound. Find the undefined behaviour;
  `cargo miri test` going clean is the grade.

## The seal is obfuscation, not secrecy

Reference solutions ship gzip-compressed and base64-encoded under `.sealed/`, so they
cannot be read by scrolling past them. Anyone who wants an answer can have it, and that is
said out loud rather than pretended otherwise.

The ladder exists to make the easy path the honest one:

```sh
cargo run -p underust -- hint   <id>          # always available
cargo run -p underust -- reveal <id>          # refuses while the tests fail
cargo run -p underust -- reveal <id> --stuck  # overrides, and records that it did
cargo run -p underust -- progress             # shows earned reveals apart from stuck ones
```

## Running the gates

```sh
task ci
```

Seven gates: formatting, clippy, every stub compiles, every sealed solution unseals and
passes, manifests validate, radar invalidations name real tasks, and no sealed text appears
in a log that will be public. `task prove` reports `N proven, M skipped` — the skips are
printed, and CI fails if their number rises.

## Licence

MIT. Ideas may be borrowed; code may not.
