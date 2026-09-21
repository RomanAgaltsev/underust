# unsafe/01-aliasing

**Mode:** SOUNDNESS — the code works. Find the undefined behaviour.

`split_at_mut_ours()` splits a slice into two mutable halves. It compiles, and most of its
tests pass. It is also **unsound**: it creates two live `&mut` that overlap the same bytes,
which the compiler is entitled to assume never happens.

```
rustup toolchain install nightly
rustup +nightly component add miri
underust test unsafe/01-aliasing
```

The harness runs `cargo +nightly miri test`. **miri's verdict is the grade** — this task
asserts nothing about undefined behaviour itself; the tool decides.

## What passing means

`cargo miri test` goes from *Undefined Behavior* to clean, **and** all four tests still
pass. Note that one of them already fails against the shipped code: the length bug and the
aliasing bug are the same bug, so fixing soundness fixes the test, while fixing only the
test still leaves miri to satisfy.

## You cannot pass by deleting the tests

`tests/grade.rs` is pinned by a digest in `task.toml`. Change it and the harness refuses to
grade, naming both the pinned and the found digest. Reformatting is fine — whitespace is
normalised before hashing — but removing an assertion is not.

## Why this task exists

This is the mode where the author asserts nothing at all. There is no sealed claim that the
code is unsound; miri reports it, and the report is checkable by anyone. That makes
SOUNDNESS the most honest mode in the gym, and the easiest to trust.

## Mastery bar

Fixing the length is half an answer. The other half: say **why** two overlapping `&mut` are
undefined behaviour even in a run where nothing is miscomputed — what the compiler is
permitted to assume about a `&mut`, and what it may do with that permission.

Then read miri's output again and say what "that tag does not exist in the borrow stack"
is describing.
