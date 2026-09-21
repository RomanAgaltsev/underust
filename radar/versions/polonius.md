# Polonius Alpha — a standing watch, not a release

Not a release entry. Polonius changes what the borrow checker *accepts*, which makes it the
richest version-diff material in the catalogue and also the biggest source of future
invalidations.

**Verified 2026-09-20** against the announcement:

- Polonius Alpha has been the **default on nightly since 2026-08-04**.
- `-Zpolonius=off` reverts to NLL. So a `borrowck` A/B needs **one toolchain and a flag**,
  not a toolchain pair — cheaper than undergo's M14b equivalent.
- Its analysis is flow-sensitive where NLL's is flow-insensitive, so it accepts programs
  NLL rejects. Two examples come straight from the announcement: a conditional reborrow
  (`let b = &mut *a; if true { b } else { a }`) and `get_mut_or_default` over a `HashMap`,
  the classic NLL Problem Case #3.
- Stabilisation is aimed at **before the end of 2026**. Worst observed compile-time
  regression is 2–3x, on rare crates.

  -> candidate | borrowck

## The standing risk

When Polonius stabilises, the A/B changes from *stable vs nightly* to *flag on vs off*, and
**any task asserting "this does not compile" is invalidated**. No `-> invalidates` line is
written here yet, because no `borrowck` task exists and gate 6 would correctly fail on a
dangling id. The moment `borrowck/01` lands, this file gains that line.

Re-check on each release from 1.99 onward.
