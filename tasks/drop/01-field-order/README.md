# drop/01-field-order

**Mode:** PREDICT — you commit a guess, then the harness measures reality.

`observe()` builds three tracked values as the fields of a struct, with two more as loose
locals on either side of it, and lets the scope end. Every value appends its label when it
drops.

```rust
let _loose_a = log.token("loose_a");
let _trio = Trio {
    first:  log.token("first"),
    second: log.token("second"),
    third:  log.token("third"),
};
let _loose_b = log.token("loose_b");
```

**Predict the order of all five labels.**

```
underust predict drop/01-field-order
# edit work/drop/01-field-order/prediction.toml
underust test drop/01-field-order
```

Your prediction is a TOML array:

```toml
order = ["...", "...", "...", "...", "..."]
```

## Mastery bar

Being right is not enough — be right for the stated reason. Before you run it, write down
which rule governs the three fields and which governs the two locals, and why they differ.
If you can only say *what* happens and not *why*, you have not passed the bar even when the
harness prints PASS.

## Why this task seals nothing

There is no answer in this repository to find. The program computes the order at run time
and the harness compares it to what you said. That is the whole mode: the author of the
task did not have to know the answer either.
