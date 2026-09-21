# own/01-no-clone

**Mode:** CONSTRAIN — make it work *without the escape hatch*.

`Ring::rotate()` moves the oldest buffer to the back of the ring, leaves an empty `String`
in its place ready for reuse, and returns what it held.

The catch: the returned `String` must be **the same allocation** the slot held — the very
bytes, not a copy of them — and you are reaching into a `Vec` through `&mut self`, so the
borrow checker will not let you simply move the value out.

```
underust test own/01-no-clone
```

## Forbidden

```
clone   to_owned   to_vec   to_string   Rc   Arc   RefCell
```

These are checked against the **parsed syntax tree** of your solution, not its text. A
comment mentioning `clone`, a string literal `"clone"`, or a variable named `cloned_flag`
will not trip the ban. A method call `.clone()` will.

## Why the ban exists

Without it this is a five-second task: clone the string and move on. The escape hatches
always work, which is exactly why an unconstrained borrow-checker exercise teaches nothing.
The constraint *is* the lesson.

Note that the tests would catch a clone anyway — `the_returned_string_is_the_original_allocation`
compares raw pointers. The ban and the test are deliberate duplicates: the test proves the
allocation survived, the ban stops you reaching that result by some other copy-shaped route.

## Mastery bar

Say why the borrow checker rejects the obvious move, and what the replacement leaves behind
in the slot while the old value is in flight. "Because the compiler said so" is not the
mechanism.
