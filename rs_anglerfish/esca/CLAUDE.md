# esca

`docs/features.md` defines every fact and decides every dispute about one.

## A fact exists only with all of these

1. Its entry in `docs/features.md`.
2. Its Rust implementation under `src/facts/`.
3. Its Python reference in `tests/reference/features.py`, at the repo root.
4. Hand-derived parameterized cases in **both** `tests/facts_<group>.rs`
   (`#[rstest]`) and `python/tests/test_facts_<group>.py` (`parametrize`),
   mirrored case for case, at least three per fact and per side, plus a
   Chess960 case for each fact `features.md` §4 marks variant-sensitive.

Cases state what the definition requires; a value read off the implementation
tests nothing. Positions are named constants with a one-line note on what they
show, and a test reads as an example of the language's own API. Helpers live in
`tests/common/mod.rs` and `python/tests/conftest.py`; tests share no mutable
state and run in parallel (`cargo test -- --test-threads 4`, pytest `-n auto`).

## Changing what a fact computes

Fix the Rust and the Python reference together, bump that group's `version` in
`src/schema.rs`, regenerate with `cargo run --release --example fixtures`, and
read the fixture diff: every changed value is a changed trained contract.
