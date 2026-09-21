# alloc/01-allocation-count

**Mode:** OPTIMIZE — make it cheaper without changing what it does.

`even_names()` uppercases the names of every even-scored entry and joins them with `", "`.
It is correct. It is also wasteful: as shipped it makes **11 heap allocations** to produce
one short string.

```rust
let filtered: Vec<&Entry> = entries.iter().filter(|e| e.score % 2 == 0).collect();
let names:    Vec<String> = filtered.iter().map(|e| e.name.clone()).collect();
let upper:    Vec<String> = names.iter().map(|n| n.to_uppercase()).collect();
let parts:    Vec<&str>   = upper.iter().map(String::as_str).collect();
parts.join(", ")
```

**Get it to 3 allocations or fewer, with identical output.**

```
underust test alloc/01-allocation-count
```

## The rule

**Both** halves must hold. The three behaviour tests must keep passing *and* the count must
come in at or under budget. Making it cheap by making it wrong is not a pass.

## Why this is graded on allocations and not on time

Allocation *requests* are deterministic: the same code makes the same number on every
machine, every run, on Windows and on Linux alike. A wall-clock threshold would assert the
properties of whatever machine happened to run it — a fast laptop passes, a loaded CI
runner fails, and neither outcome says anything about your code.

So there is no warm-up here, no statistics and no tolerance. The number is the number.

## Mastery bar

Name each of the 11 allocations before you change anything. Then say which of them are
*intermediate* — allocated only to be read once and dropped — and why the compiler cannot
remove them for you.
