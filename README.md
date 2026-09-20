# underust

A **Rust internals gym**: small, self-contained tasks whose answers are *computed by an
instrument* rather than asserted by an author. Fork it, solve tasks in a gitignored
workspace, and the harness tells you whether reality agrees with you.

> **Status: under construction.** The harness is being built; see `ROADMAP.md`.
> This README is expanded in full before the repository goes public.

## The mastery bar

**Predict the behaviour, then cite the mechanism.** A task is not passed by producing output
that happens to be correct. It is passed by having said, in advance, what the machine would
do — and being right for the stated reason.

## The organising rule

> A track ships only when an instrument computes its answers.

A drop log, a counting allocator, `Rc::strong_count`, `miri`, `loom` and rustc's own
diagnostics are not opinions. Where no instrument can settle the question, the task does not
ship.

## Getting started

```
rustup toolchain install 1.98.1
cargo run -p underust -- doctor
```

`doctor` reports which tracks this machine can grade and prints the exact command that fixes
each gap.

## The seal is obfuscation, not secrecy

Reference solutions ship gzip-compressed and base64-encoded so they cannot be read by
scrolling past them. Anyone who wants the answer can have it, and that is said out loud
rather than pretended otherwise. The `hint` → `reveal` gate exists to make the easy path the
honest one, not to stop a determined reader.

## Licence

MIT. Ideas may be borrowed; code may not.
