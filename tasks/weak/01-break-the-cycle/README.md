# weak/01-break-the-cycle

**Mode:** BUILD — implement it; the tests decide.

A two-level tree. A parent owns its children, and each child needs to reach back up to its
parent. Written naively, that is a reference cycle: parent and child keep each other alive
forever and neither is ever dropped.

Implement `build()` and `parent_value_of()` so that **both** of these hold:

- a child can read its parent's value while the parent is alive
- dropping the last strong handle to the parent **actually drops it**

```
underust test weak/01-break-the-cycle
```

## What the tests check

- `the_child_can_reach_its_parent` — the upward link works
- `the_parent_is_not_kept_alive_by_its_child` — after `drop(parent)`, a `Weak` to it fails
  to upgrade, and the child's upward link reports `None`
- `the_parent_owns_its_child` — the downward link is still ownership

The second test is the one with teeth. A cycle passes the first and third and fails only
this one, which is why it exists.

## Mastery bar

Say which direction owns and which observes, and why that choice and not the reverse.
Then say what `upgrade()` returning `None` proves about the memory, not just about the API.
