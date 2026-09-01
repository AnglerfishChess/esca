# Names for the position-facts library

`esca` was chosen; the candidates below are the record of the choice.

Availability checked on 2026-08-31 against `crates.io/api/v1/crates/<name>`
and `pypi.org/pypi/<name>/json`; 404 on both means free. The crate name and
the Python module name should match (hyphens become underscores in Python).

| Name | crates.io | PyPI | Rationale |
|---|---|---|---|
| `esca` | free | free | The anglerfish's luminous lure: the small thing that lights up what is in front of it. Short, on-theme, no chess-library echo. |
| `illicium` | free | free | The fin-ray that carries the esca. Same theme, longer and more distinctive; unlikely to ever collide. |
| `photophore` | free | free | A light-producing organ. Says "shows you what is there" without saying "chess"; reads as a general library. |
| `bathyal` | free | free | The deep-sea zone the anglerfish lives in. Atmospheric, ties to the project, carries no meaning to argue with. |
| `chessight` | free | free | chess + sight. Says the contract: what a position lets you see at a glance. |
| `boardsense` | free | free | What a strong player reads off a board immediately — passers, open files, hanging pieces. Plain English, searchable. |
| `chess-facts` | free | free | Literal and discoverable: facts about a chess position. Zero cleverness, best for someone searching crates.io. |
| `oneply` | free | free | Names the boundary exactly: everything derivable within one ply. The contract as the name. |

Taken, checked and rejected: `lure`, `lantern`, `glimmer` (crates.io and
PyPI), `salience` (crates.io).

Two notes for the choice:

- A name of the form `cozy-*`, `*-chess-facts` or anything echoing an existing
  chess library would read as a fork or a companion crate. None of the above
  do.
- `esca`, `illicium`, `photophore` and `bathyal` do not say "chess"; the crate
  description and keywords carry that. `chessight`, `boardsense`,
  `chess-facts` and `oneply` do say it, at the cost of being less distinctive.
