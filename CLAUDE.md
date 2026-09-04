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

## Every other module, the same way

Anything esca exposes (rules, text, PGN, UCI, readers) is covered by
parameterized cases in both languages, mirrored case for case, hand-derived
and readable as usage examples, independent and parallel-safe.

## Changing what a fact computes

Fix the Rust and the Python reference together, bump that group's `version` in
`src/schema.rs`, regenerate with `cargo run --release --example fixtures`, and
read the fixture diff: every changed value is a changed trained contract.

## Repo layout and commands

The crate is the repository: `src/`, `tests/` and `benches/` at the root,
`data/` for the bundled ECO catalogue, `python/` for the maturin project
(`python/esca` the package, `python/tests` its suite), `docs/` for the spec.

```sh
uv sync --all-groups            # builds the extension in place, editable
uv run --no-sync pytest
cargo test --all-features
```

Gates, all clean before a commit: `cargo test --all-features`,
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
`uv run --no-sync pytest`, `uvx ruff check .`, `uvx ruff format --check .`,
`uvx pyrefly check`.

Every agent works in its own worktree (`git worktree add ./<task> -b <task>
main`), never in `main` and never one shared with another agent.
