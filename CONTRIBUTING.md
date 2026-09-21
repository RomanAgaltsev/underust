# Contributing to underust

## The bar a proposed task has to clear

> A track ships only when an instrument computes its answers.

Before anything else, say what *measures* your task. A drop log, a counting allocator,
`Rc::strong_count`, `trybuild`'s committed stderr, `miri`'s verdict, `loom`'s enumeration —
something that produces the answer at run time rather than an answer you wrote down.

If the only thing that can settle the question is the author's opinion, it is not a task
here yet. That is not a comment on its value; REVIEW and DESIGN exist for exactly that kind
of material and are graded by a human against a rubric.

## Two rules that are not obvious

**Ideas may be borrowed; code may not.** If a task came from somewhere, record it in
`inspired_by` in `task.toml`. Do not copy source from another project, however small.

**Task IDs are permanent.** An id is part of the SemVer contract, alongside the CLI surface
and the `task.toml` schema. Renaming one breaks every solver's local progress, so ids are
chosen once and kept.

## What an answer may assert

`answer_stability` in `task.toml` is either `invariant` or `observed`, and it is not
decoration.

- **`invariant`** — guaranteed by the language or by library semantics. Drop order,
  allocation *request* counts, `Rc` strong and weak counts, `size_of` under `#[repr(C)]`,
  miri verdicts, loom enumerations.
- **`observed`** — true when measured, not promised. Anything under `#[repr(Rust)]`, any
  generated assembly, anything from a `-Z` flag. These must pin `requires` to an exact
  toolchain and say so in the task's README.

Getting this wrong is the single most likely way for a task to rot. `#[repr(Rust)]` layout
is explicitly unspecified and the compiler actively reorders fields.

**Never grade on wall-clock time.** A threshold asserts the properties of whatever machine
ran it. Use a deterministic cost, or compare against a baseline measured in the same run.

## Adding a task

```
tasks/<track>/<NN-slug>/
  Cargo.toml        # workspace member, publish = false
  task.toml         # id, track, mode, points, requires, hint
  README.md         # the prompt, and the mastery bar
  src/lib.rs        # the stub, or the complete program -- see below
  tests/grade.rs    # the grader
```

The stub convention depends on the mode. BUILD, CONSTRAIN and OPTIMIZE ship `todo!()` or
working-but-wasteful code. **PREDICT ships a complete, working program** — the solver writes
a prediction, not code. **SOUNDNESS ships working but unsound code**, because `todo!()`
would delete the very undefined behaviour the task is about.

Then:

```sh
cargo run -p underust -- validate
cargo run -p underust -- test <id>
```

A task with a reference solution needs it sealed. Author it **outside the repository**, seal
it, and delete the source — `.sealed/` holds the only copy:

```sh
cargo run -p underust -- seal <id> --from /tmp/my-solution
```

where that directory contains `EXPLANATION.md` and `files/` mirroring the task layout. The
explanation must name the *mechanism*, not just show the code; that is what `reveal` prints,
and it is the whole value of the seal.

For a SOUNDNESS task, pin the grader afterwards:

```sh
cargo run -p underust -- digest <id>   # paste into test_digest in task.toml
```

## Before opening a PR

```sh
task ci
```

All seven gates. Title the PR conventionally — `feat(tasks): ...` for anything that adds
tasks, because release-please bumps the version on `feat` and `fix` only, and a `chore:`
title lands in the changelog without a release.

## One habit this repository cares about

**Prove a check can fail before trusting it.** Every gate here has been watched going red:
the reveal gate refusing, the ban rejecting a `.clone()`, miri rejecting the unsound stub,
the leak check catching sealed text, the seal's golden bytes catching a changed timestamp.
A green check nobody has seen fail is not evidence of anything.

If you add a gate, break it on purpose once, and say in the PR what you saw.
