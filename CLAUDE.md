# esca

A fact is declared once, in the `facts_group!` block that defines its group in
`src/facts/mod.rs`: the field, its type and the doc comment that is its
meaning. `docs/features.md` is generated from those blocks
(`cargo run --example catalogue`) and a test fails when it is stale.

## A fact exists only with all of these

1. Its field, type and doc comment in the group's `facts_group!` block.
2. Its computation under `src/facts/`, written by colour, never by
   side-to-move.
3. Hand-derived parameterized cases in **both** `tests/facts_<group>.rs`
   (`#[rstest]`) and `python/tests/test_facts_<group>.py` (`parametrize`),
   mirrored case for case, at least three per fact and per colour, a case with
   Black to move for anything a mover's view could have flipped, and a Chess960
   case for anything that reads the back rank.
4. `docs/features.md` regenerated.

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

Fix the computation and the cases that pin it in both languages, regenerate
`docs/features.md`, and say in `CHANGELOG.md` what now answers differently: a
consumer reads these by name, so a changed meaning is a changed contract.

## Repo layout and commands

The crate is the repository: `src/`, `tests/`, `benches/` and `examples/` at
the root, `data/` for the bundled ECO catalogue, `python/` for the maturin
project (`python/esca` the package, `python/tests` its suite), `docs/` for the
spec, `mcp/` for the MCP server.

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
