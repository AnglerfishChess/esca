"""The tools, the facts-schema resource and the analysis prompt.

Every tool reads: nothing on the machine is written and nothing off it is
fetched. An answer is a JSON object, and a call that names no position, no
move or no game answers `{"error": {...}}` carrying the evidence to fix it.
"""

import re
from collections.abc import Callable
from functools import wraps
from typing import Annotated, Any

import esca
from mcp.server.fastmcp import FastMCP
from mcp.types import ToolAnnotations
from pydantic import Field

from chess_esca_mcp import facts as facts_module
from chess_esca_mcp import rendering as render
from chess_esca_mcp.positions import (
    ChessInputError,
    parse_move,
    pgn_games,
    read_game,
    read_pgn,
    variant_label,
)

mcp = FastMCP("chess-esca")

#: The categories `legal_moves` sorts a position's moves into.
MOVE_CATEGORIES = ("captures", "checks", "castling", "promotions", "en_passant", "quiet")

#: Both colours, as esca spells them and as the answers do.
SIDES = (("w", "white"), ("b", "black"))

_UCI = re.compile(r"^([a-h][1-8])([a-h][1-8])([qrbnQRBN])?$")
_SAN_DESTINATION = re.compile(r"([a-h][1-8])(?:=[QRBNqrbn])?[+#]?$")

Fen = Annotated[
    str | None,
    Field(description="The position as FEN or EPD; the variant's start position when omitted."),
]
Moves = Annotated[
    list[str] | None,
    Field(
        description=(
            "Moves played from `fen`, each SAN ('Nf3') or UCI ('g1f3'). Repetition and "
            "fifty-move claims are only visible with the moves that led here."
        )
    ),
]
Pgn = Annotated[
    str | None,
    Field(description="A whole game as PGN, instead of `fen` and `moves`."),
]
Variant = Annotated[str, Field(description="The rules: 'classic' or 'chess960'.")]


def reading(title: str) -> ToolAnnotations:
    """What every tool here is: it reads, and it reaches nothing outside."""
    return ToolAnnotations(
        title=title,
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )


def answering(function: Callable[..., dict[str, Any]]) -> Callable[..., dict[str, Any]]:
    """Turns an input esca will not take into an error object, not a traceback."""

    @wraps(function)
    def answer(*args: Any, **kwargs: Any) -> dict[str, Any]:
        try:
            return function(*args, **kwargs)
        except ChessInputError as error:
            return error.content()
        except (ValueError, OSError) as error:
            return {"error": {"kind": "invalid_input", "message": str(error)}}

    return answer


def _opening(found: esca.openings.Opening | None) -> dict[str, str] | None:
    """An ECO entry as JSON."""
    return None if found is None else {"eco": found.eco, "name": found.name}


def _material(facts: esca.Facts) -> dict[str, Any]:
    """What each side has on the board, and what it comes to."""
    counted = facts_module.group_facts(facts, "material")
    white = facts.side("w")
    return {
        "white": counted["count"]["white"],
        "black": counted["count"]["black"],
        "value": counted["value"],
        "non_pawn_value": counted["non_pawn_value"],
        "balance": facts.material.value[white] - facts.material.value[1 - white],
        "phase": facts.material.phase,
        "insufficient": counted["insufficient"],
    }


def _status(game: esca.Game, history_known: bool) -> dict[str, Any]:
    """Whether the game is over, how, and what either side could claim now."""
    outcome = game.outcome()
    mover = render.colour(game.position.side_to_move)
    winner = render.other(mover) if outcome == "checkmate" else None
    if outcome == "checkmate":
        result = "1-0" if winner == "white" else "0-1"
    elif outcome is not None:
        result = "1/2-1/2"
    else:
        result = "*"
    return {
        "state": outcome or "in_progress",
        "winner": winner,
        "result": result,
        "claims": list(game.claims()),
        "history_known": history_known,
        **render.draw_status(game),
    }


def _check(position: esca.Position) -> dict[str, Any]:
    """Who is giving check to the side to move, and from where."""
    checkers = position.checkers()
    return {
        "in_check": position.in_check,
        "double_check": len(checkers) > 1,
        "king": position.king_of(position.side_to_move),
        "checkers": render.units(position, checkers),
    }


def _move_content(game: esca.Game, mv: esca.Move, move_facts: esca.MoveFacts) -> dict[str, Any]:
    """One legal move: its text, and what it does on the board."""
    return {
        "san": game.move_to_san(mv),
        "uci": game.move_to_uci(mv),
        "kind": mv.kind,
        "mover": render.role(move_facts.mover),
        "capture": mv.is_capture,
        "victim": None if move_facts.victim is None else render.role(move_facts.victim),
        "promotion": None if mv.promotion is None else render.role(mv.promotion),
        "gives_check": move_facts.gives_check,
        "see": move_facts.see,
    }


def _categorise(moves: list[dict[str, Any]]) -> dict[str, list[str]]:
    """The SAN of each move, under every category it belongs to."""
    grouped: dict[str, list[str]] = {name: [] for name in MOVE_CATEGORIES}
    for move in moves:
        if move["capture"]:
            grouped["captures"].append(move["san"])
        if move["gives_check"]:
            grouped["checks"].append(move["san"])
        if move["kind"] == "castling":
            grouped["castling"].append(move["san"])
        if move["promotion"] is not None:
            grouped["promotions"].append(move["san"])
        if move["kind"] == "en_passant":
            grouped["en_passant"].append(move["san"])
        if move["kind"] == "quiet":
            grouped["quiet"].append(move["san"])
    return grouped


def _line(game: esca.Game) -> list[str]:
    """The moves of `game` in SAN, read from its own start position."""
    replay = esca.Game.from_position(game.start_position, variant=game.variant)
    written = []
    for mv in game.moves:
        written.append(replay.move_to_san(mv))
        replay.play(mv)
    return written


@mcp.tool(name="position", annotations=reading("Read a chess position"))
@answering
def position(fen: Fen = None, moves: Moves = None, pgn: Pgn = None, variant: Variant = "classic") -> dict:
    """The whole state of a chess position, read by the rules.

    Answers in one call: whose turn it is, whether that side is in check and
    from where, whether the game is over and how, which draws are claimable
    and on what evidence, how many legal moves there are, the opening name if
    the position has one, the material on each side, both castlings of both
    colours with every obstacle at once, the en-passant capture on offer, the
    pins and skewers on the board, and the ending it is where the material
    makes one. `prose` gathers every sentence the answer carries.

    Args:
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI. Repetition and fifty-move
            claims are only visible when the moves that led here are given.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.

    Returns:
        dict: `fen`, `epd`, `variant`, `side_to_move`, `fullmove_number`,
        `halfmove_clock`, `ply`, `line`, `status`, `check`,
        `legal_move_count`, `opening`, `material`, `castling`, `en_passant`,
        `pins`, `skewers`, `ending` where the material is one, and `prose`,
        every sentence the answer carries — or `error` when the input names no
        position.
    """
    game = read_game(fen, moves, pgn, variant)
    board = game.position
    facts = game.facts()
    named = game.ending()
    answer: dict[str, Any] = {
        "fen": board.fen,
        "epd": board.epd,
        "variant": variant_label(game.variant),
        "side_to_move": render.colour(board.side_to_move),
        "fullmove_number": board.fullmove_number,
        "halfmove_clock": board.halfmove_clock,
        "ply": game.ply,
        "line": _line(game),
        "status": _status(game, history_known=bool(moves or pgn)),
        "check": _check(board),
        "legal_move_count": len(game.legal_moves()),
        "opening": _opening(esca.openings.lookup(board)),
        "material": _material(facts),
        "castling": render.castling(board),
        "en_passant": render.en_passant(board),
        "pins": {name: [render.pin(item) for item in board.pins(letter)] for letter, name in SIDES},
        "skewers": {name: [render.skewer(item) for item in board.skewers(letter)] for letter, name in SIDES},
    }
    if named.class_.name != render.NOT_AN_ENDING:
        answer["ending"] = render.ending_summary(named)
    answer["prose"] = render.gathered(answer)
    return answer


@mcp.tool(name="legal_moves", annotations=reading("List the legal moves"))
@answering
def legal_moves(
    fen: Fen = None,
    moves: Moves = None,
    pgn: Pgn = None,
    variant: Variant = "classic",
    categories: Annotated[bool, Field(description="Also group the moves by what they do.")] = True,
) -> dict:
    """Every legal move of a position, in SAN and in UCI.

    Each move carries what it does — the role that moves, whether it captures
    and what, whether it gives check, what it promotes to, and the static
    exchange it starts. With `categories`, the same moves are also grouped as
    captures, checks, castling, promotions, en passant and quiet.

    Args:
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.
        categories: group the moves as well as listing them.

    Returns:
        dict: `fen`, `side_to_move`, `count`, `moves` — each `san`, `uci`,
        `kind`, `mover`, `capture`, `victim`, `promotion`, `gives_check`,
        `see` — and `by_category` when asked for; or `error`.
    """
    game = read_game(fen, moves, pgn, variant)
    listed = [_move_content(game, item.move, item.facts) for item in game.annotated_moves()]
    listed.sort(key=lambda move: move["san"])
    answer: dict[str, Any] = {
        "fen": game.position.fen,
        "side_to_move": render.colour(game.position.side_to_move),
        "count": len(listed),
        "moves": listed,
    }
    if categories:
        answer["by_category"] = _categorise(listed)
    return answer


def _castling_reasons(game: esca.Game, text: str) -> tuple[list[dict[str, Any]], list[str]]:
    """Everything standing in the way of the castling `text` names, and the
    sentence the whole castling reads as."""
    wing = "short" if text.replace("0", "O").upper() == "O-O" else "long"
    answer = render.castling_wing(game.position, render.colour(game.position.side_to_move), wing)
    return answer["obstacles"] or [{"reason": "no_such_legal_move", "castling": answer}], answer["prose"]


def _en_passant_reasons(board: esca.Position, destination: str, origin: str | None) -> list[dict[str, Any]]:
    """What forbids the en-passant captures of `destination`, if it is on offer."""
    offer = board.en_passant_status()
    if offer.target != destination:
        return []
    return [
        {"capture": "en_passant", "origin": capture.origin, **render.ep_obstacle(capture.forbidden_by)}
        for capture in offer.captures
        if capture.forbidden_by is not None and origin in (None, capture.origin)
    ]


def _origin_reasons(game: esca.Game, origin: str, destination: str, promotion: str | None) -> list[dict]:
    """Why the unit on `origin` cannot go to `destination`."""
    board = game.position
    mover = board.side_to_move
    reasons: list[dict[str, Any]] = []
    unit = render.unit(board, origin)
    if unit is None:
        return [{"reason": "empty_origin", "square": origin}]
    if unit["colour"] != render.colour(mover):
        return [
            {"reason": "not_the_side_to_move", "unit": unit, "side_to_move": render.colour(mover)},
        ]
    target = render.unit(board, destination)
    if target is not None and target["colour"] == unit["colour"]:
        reasons.append({"reason": "destination_held_by_own_unit", "unit": target})
    blockers = board.between(origin, destination) & board.occupied
    if blockers:
        reasons.append({"reason": "path_blocked", "occupants": render.units(board, blockers)})
    reasons.extend(_en_passant_reasons(board, destination, origin))
    for item in board.pins(mover):
        if item.pinned == origin and destination not in item.ray and destination != item.pinner:
            reasons.append({"reason": "pinned", **render.pin(item)})
    if board.in_check:
        reasons.append(
            {
                "reason": "king_in_check",
                "king": board.king_of(mover),
                "checkers": render.units(board, board.checkers()),
            }
        )
    if unit["role"] == "pawn" and promotion is None and destination[1] in ("1", "8"):
        reasons.append(
            {
                "reason": "promotion_not_named",
                "hint": "a pawn reaching the last rank must name what it becomes, as in e7e8q",
                "promotion_roles": [render.role(item) for item in game.variant.promotion_roles],
            }
        )
    if not reasons:
        reasons.append({"reason": "not_a_move_of_this_unit", "unit": unit})
    return reasons


def _destination_reasons(game: esca.Game, text: str) -> list[dict[str, Any]]:
    """Why no unit answers the SAN `text`, as far as its destination says."""
    board = game.position
    found = _SAN_DESTINATION.search(text)
    reasons: list[dict[str, Any]] = []
    if board.in_check:
        reasons.append(
            {
                "reason": "king_in_check",
                "king": board.king_of(board.side_to_move),
                "checkers": render.units(board, board.checkers()),
            }
        )
    if found is None:
        reasons.append({"reason": "unreadable_move_text", "text": text})
        return reasons
    destination = found.group(1)
    reasons.extend(_en_passant_reasons(board, destination, None))
    reasons.append(
        {
            "reason": "no_legal_move_to_destination",
            "destination": destination,
            "occupant": render.unit(board, destination),
            "legal_moves_to_destination": sorted(
                game.move_to_san(mv) for mv in game.legal_moves() if mv.destination == destination
            ),
        }
    )
    return reasons


def _illegal_reasons(game: esca.Game, text: str) -> tuple[list[dict[str, Any]], list[str] | None, list[str]]:
    """Every reason `text` names no legal move here, what the named unit could
    play instead, and any sentence that belongs to the whole answer."""
    stripped = text.strip()
    if stripped.replace("0", "O").upper() in ("O-O", "O-O-O"):
        reasons, said = _castling_reasons(game, stripped)
        return reasons, None, said
    uci = _UCI.match(stripped)
    if uci is None:
        return _destination_reasons(game, stripped), None, []
    origin, destination, promotion = uci.groups()
    from_origin = sorted(game.move_to_san(mv) for mv in game.legal_moves() if mv.origin == origin)
    return _origin_reasons(game, origin, destination, promotion), from_origin, []


@mcp.tool(name="explain_move", annotations=reading("Explain one move"))
@answering
def explain_move(
    move: Annotated[str, Field(description="The move, SAN ('exd6', 'O-O') or UCI ('e5d6', 'e1g1').")],
    fen: Fen = None,
    moves: Moves = None,
    pgn: Pgn = None,
    variant: Variant = "classic",
) -> dict:
    """Whether one move is legal, and what it does or what forbids it.

    An illegal move answers with every reason that applies at once — no
    castling right, a rook that is not there, a king in check, an attacked or
    a blocked path, a pinned unit, an occupied destination, a promotion left
    unnamed — each carrying the squares it was read off. A legal move answers
    with what it changes: the capture, the check, the castling or en passant,
    the promotion, the position after it, and the draws either side could
    claim once it is played.

    Args:
        move: the move, SAN or UCI.
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.

    Returns:
        dict: `legal`, `move`, `fen`, `prose`, and either `reasons` — a list of
        `reason`-and-evidence objects — with `legal_moves`, or the move's own
        text with `effects`, `after` and `claims_after`; or `error`.
    """
    game = read_game(fen, moves, pgn, variant)
    board = game.position
    mv = parse_move(game, move)
    if mv is None:
        reasons, from_origin, said = _illegal_reasons(game, move)
        answer: dict[str, Any] = {
            "legal": False,
            "move": move,
            "fen": board.fen,
            "side_to_move": render.colour(board.side_to_move),
            "reasons": reasons,
            "legal_moves": sorted(game.move_to_san(item) for item in game.legal_moves()),
        }
        if from_origin is not None:
            answer["legal_moves_from_origin"] = from_origin
        answer["prose"] = said + [line for line in render.gathered(answer) if line not in said]
        return answer

    move_facts = next(item.facts for item in game.annotated_moves() if item.move == mv)
    content = _move_content(game, mv, move_facts)
    claims = [render.draw(claim) for claim in game.claims_after(mv)]
    game.play(mv)
    played: dict[str, Any] = {
        "legal": True,
        "move": move,
        "fen": board.fen,
        **content,
        "effects": {
            "gives_check": move_facts.gives_check,
            "gives_safe_check": move_facts.gives_safe_check,
            "is_castling": mv.is_castling,
            "is_en_passant": mv.is_en_passant,
            "promotion": content["promotion"],
            "captures": content["victim"],
            "static_exchange": move_facts.see,
            "is_safe": move_facts.is_safe,
            "leaves_unit_hanging": move_facts.leaves_unit_hanging,
            "blocks_check": move_facts.blocks_check,
            "escapes_attack": move_facts.escapes_attack,
            "gives_discovered_attack": move_facts.gives_discovered_attack,
        },
        "after": {
            "fen": game.position.fen,
            "status": _status(game, history_known=bool(moves or pgn)),
            "check": _check(game.position),
            "legal_move_count": len(game.legal_moves()),
        },
        "claims_after": claims,
    }
    played["prose"] = render.gathered(played)
    return played


@mcp.tool(name="facts", annotations=reading("Read the named facts of a position"))
@answering
def facts(
    fen: Fen = None,
    moves: Moves = None,
    pgn: Pgn = None,
    variant: Variant = "classic",
    groups: Annotated[
        list[str] | None,
        Field(
            description=(
                "Which fact groups to read. Every group the esca://facts-schema resource lists is "
                "allowed; `placement` and `planes` are left out unless asked for."
            )
        ),
    ] = None,
) -> dict:
    """The named facts esca reads off a position, group by group.

    Each value is labelled by its own name and by the side it belongs to,
    never a bare vector. The groups are `state`, `history`, `material`,
    `pawns`, `pieces`, `king`, `mobility`, `attacks`, `exchange`, `threats`,
    `tactics`, `endgame`, and the two raw ones, `placement` and `planes`.

    Args:
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI. `history` needs them.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.
        groups: the groups to read; all but `placement` and `planes` by default.

    Returns:
        dict: `fen`, `schema_id`, `schema_version`, `perspective`, `groups`
        keyed by group name, `groups_returned`, `groups_available` — or
        `error`.
    """
    if groups:
        unknown = facts_module.unknown_groups(groups)
        if unknown:
            raise ChessInputError(
                "unknown_fact_group",
                f"no group is named {', '.join(repr(name) for name in unknown)}",
                hint="read esca://facts-schema for the groups and what each holds",
                groups_available=list(facts_module.GROUPS),
            )
    game = read_game(fen, moves, pgn, variant)
    return {"fen": game.position.fen, **facts_module.facts_content(game.facts(), groups)}


@mcp.tool(name="ending", annotations=reading("Name the ending"))
@answering
def ending(fen: Fen = None, moves: Moves = None, pgn: Pgn = None, variant: Variant = "classic") -> dict:
    """The ending the material makes, what theory says it is, and how it is won.

    An ending is a position where neither side holds more than two pieces, a
    piece being anything that is neither a king nor a pawn; above that the
    class is `not_an_ending` and the signature still says what is on the board.
    The signature writes the stronger side first — `KRPvKR` — the class names
    the material alone, and the verdict names the colour it favours. The
    technique is the named method the ending is played by: the box method, the
    Lucena position, the opposition, the wrong bishop. The `evidence` holds the
    position-specific facts the verdict and the technique were read off, each
    behind the reason it belongs to.

    This is the books' answer for the material, adjusted by those facts. It is
    not a search, and it does not know whether this particular position is won.

    Args:
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.

    Returns:
        dict: `fen`, `is_ending`, `signature`, `class`, `verdict` (`kind` and
        `colour`), `technique`, `evidence` (`pawn`, `bishops`, `opposition`)
        and `prose` — or `error`.
    """
    game = read_game(fen, moves, pgn, variant)
    named = game.ending()
    return {
        "fen": game.position.fen,
        "is_ending": named.class_.name != render.NOT_AN_ENDING,
        **render.ending(named),
    }


@mcp.tool(name="opening", annotations=reading("Name the opening"))
@answering
def opening(fen: Fen = None, moves: Moves = None, pgn: Pgn = None, variant: Variant = "classic") -> dict:
    """The ECO code and name of a position, and of the line that reached it.

    ECO is the Encyclopaedia of Chess Openings, whose volume letter and index
    name a known opening. The catalogue is keyed by position, so a line that
    transposes into a named position is named, and a line that has left the
    book keeps the last name it reached, which is what `reached` reports.

    Args:
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI. `reached` needs them.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.

    Returns:
        dict: `fen`, `at_position` (`eco` and `name`, or null), `reached`,
        `line`, `ply`, `named_positions_in_catalogue` — or `error`.
    """
    game = read_game(fen, moves, pgn, variant)
    return {
        "fen": game.position.fen,
        "at_position": _opening(esca.openings.lookup(game.position)),
        "reached": _opening(game.opening()),
        "line": _line(game),
        "ply": game.ply,
        "named_positions_in_catalogue": esca.openings.count(),
    }


@mcp.tool(name="book_moves", annotations=reading("Look a position up in an opening book"))
@answering
def book_moves(
    book: Annotated[str, Field(description="Path to a Polyglot opening book, a `.bin` file.")],
    fen: Fen = None,
    moves: Moves = None,
    pgn: Pgn = None,
    variant: Variant = "classic",
    limit: Annotated[int, Field(ge=1, le=200, description="How many entries to return, heaviest first.")] = 20,
) -> dict:
    """The moves a Polyglot opening book holds for a position.

    Polyglot is the opening-book format whose 16-byte entries are keyed by a
    hash of the position, so a book written by any program that speaks it can
    be read here. No book is bundled: `book` names one of your own.

    Args:
        book: path to a `.bin` Polyglot book.
        fen: the position as FEN or EPD; the start position when omitted.
        moves: moves played from it, SAN or UCI.
        pgn: a whole game instead of `fen` and `moves`.
        variant: 'classic' or 'chess960'.
        limit: how many entries to return, heaviest first.

    Returns:
        dict: `book`, `entries_in_book`, `polyglot_key`, `fen`, `count`,
        `moves` — each `san`, `uci`, `weight`, `share`, `learn` — and `best`;
        or `error` when the book cannot be read.
    """
    game = read_game(fen, moves, pgn, variant)
    try:
        opened = esca.polyglot.Book(book)
    except (OSError, ValueError) as error:
        raise ChessInputError(
            "book_unreadable",
            f"{book} could not be read as a Polyglot book: {error}",
            hint=(
                "point `book` at a `.bin` Polyglot book, whose length is a multiple of 16 bytes; "
                "esca.polyglot.download(url, path) fetches one"
            ),
            book=book,
        ) from None
    entries = opened.entries(game.position, variant=game.variant)
    total = sum(entry.weight for entry in entries) or 1
    heaviest = sorted(entries, key=lambda entry: -entry.weight)[:limit]
    return {
        "book": book,
        "entries_in_book": len(opened),
        "polyglot_key": f"{game.position.polyglot_key:016x}",
        "fen": game.position.fen,
        "count": len(entries),
        "moves": [
            {
                "san": game.move_to_san(entry.move),
                "uci": game.move_to_uci(entry.move),
                "weight": entry.weight,
                "share": round(entry.weight / total, 4),
                "learn": entry.learn,
            }
            for entry in heaviest
        ],
        "best": None if not heaviest else game.move_to_san(heaviest[0].move),
    }


@mcp.tool(name="pgn", annotations=reading("Read a game as PGN"))
@answering
def pgn(
    text: Annotated[str, Field(description="The game as PGN: tag pairs, a blank line, the movetext.")],
    index: Annotated[int, Field(ge=0, description="Which game of the source to read, from 0.")] = 0,
) -> dict:
    """A PGN game read into its headers, its moves and where it ended up.

    PGN is Portable Game Notation, the text a game is saved and shared in.
    Comments, numeric glyphs and variation counts are kept where the source
    has them; the moves reported are the mainline.

    Args:
        text: the PGN source; it may hold more than one game.
        index: which game to read, counting from 0.

    Returns:
        dict: `games_in_source`, `index`, `headers`, `comment`, `result`,
        `variant`, `start_fen`, `moves` — each `san`, `uci`, `nags`,
        `comment_before`, `comment_after`, `variations` — `ply`, `final_fen`,
        `opening`, `status`; or `error`.
    """
    source = pgn_games(text)
    parsed = read_pgn(text, index)
    try:
        played = parsed.game()
    except ValueError as error:
        raise ChessInputError(
            "unplayable_pgn",
            f"the movetext does not play out: {error}",
            hint="every move must be legal in the position it is written for",
        ) from None
    replay = esca.Game.from_position(parsed.start_position, variant=parsed.variant)
    listed = []
    for node in parsed.mainline():
        listed.append(
            {
                "san": node.san,
                "uci": replay.move_to_uci(node.move),
                "nags": list(node.nags),
                "comment_before": node.comment_before,
                "comment_after": node.comment_after,
                "variations": len(node.variations),
            }
        )
        replay.play(node.move)
    return {
        "games_in_source": len(source),
        "index": index,
        "headers": dict(parsed.headers),
        "comment": parsed.comment,
        "result": parsed.result,
        "variant": variant_label(parsed.variant),
        "start_fen": parsed.start_position.fen,
        "moves": listed,
        "ply": len(listed),
        "final_fen": played.position.fen,
        "opening": _opening(played.opening()),
        "status": _status(played, history_known=True),
    }


@mcp.tool(name="to_pgn", annotations=reading("Write a game as PGN"))
@answering
def to_pgn(
    moves: Annotated[list[str], Field(description="The moves of the game, each SAN ('Nf3') or UCI ('g1f3').")],
    fen: Fen = None,
    variant: Variant = "classic",
    headers: Annotated[
        dict[str, str] | None,
        Field(description="Tag pairs to set, such as White, Black, Event, Date."),
    ] = None,
    result: Annotated[
        str | None,
        Field(description="The termination marker: '1-0', '0-1', '1/2-1/2' or '*'."),
    ] = None,
) -> dict:
    """A list of moves written out as a PGN game.

    Args:
        moves: the moves, SAN or UCI.
        fen: the start position; the variant's own when omitted.
        variant: 'classic' or 'chess960'.
        headers: tag pairs to set on the game.
        result: the termination marker; read from the final position when
            omitted.

    Returns:
        dict: `pgn`, `final_fen`, `ply`, `result`, `opening` — or `error` when
        a move is not legal where it stands.
    """
    game = read_game(fen, moves, None, variant)
    written = game.to_pgn()
    for name, value in (headers or {}).items():
        written.set_header(name, value)
    written.result = result if result is not None else _status(game, history_known=True)["result"]
    return {
        "pgn": written.to_string(),
        "final_fen": game.position.fen,
        "ply": game.ply,
        "result": written.result,
        "opening": _opening(game.opening()),
    }


@mcp.resource(
    "esca://facts-schema",
    name="esca facts schema",
    description="The fact groups and feature names of the esca schema, with its id and width.",
    mime_type="application/json",
)
def facts_schema() -> dict:
    """The schema the `facts` tool answers from.

    Every group with its version, its width and its offset in the encoded row,
    and the names of the features it writes.
    """
    schema = esca.SCHEMA
    named: dict[str, list[str]] = {name: [] for name in schema.group_names}
    for group, feature in schema.features_for(esca.CLASSIC):
        named[group].append(feature)
    return {
        "schema_id": schema.id,
        "version": schema.semver,
        "width": schema.width,
        "feature_count": schema.feature_count,
        "groups": [dict(group, features=named[group["name"]]) for group in schema.groups()],
    }


@mcp.prompt(
    name="analyse-position",
    description="Read a chess position with the esca tools and report what is true about it.",
)
def analyse_position(
    fen: Annotated[str, Field(description="The position to analyse, as FEN.")],
    moves: Annotated[str, Field(description="The moves that led here, space separated.")] = "",
) -> str:
    """The instructions for reading one position with these tools."""
    history = (
        f"The moves that led here, in order: {moves}. Pass them as `moves` so that repetition "
        "and fifty-move claims can be seen.\n"
        if moves
        else ""
    )
    return (
        f"Analyse this chess position: {fen}\n{history}\n"
        "Work from the rules, not from memory:\n"
        "1. Call `position` first. It answers whose turn it is, the check and game status, the "
        "claimable draws with their evidence, the material, both castlings with every obstacle, "
        "the en-passant offer, and the pins and skewers.\n"
        "2. Call `legal_moves` to see what can actually be played, grouped as captures, checks, "
        "castling, promotions and quiet moves.\n"
        "3. Call `explain_move` for any move you are unsure of; an illegal one comes back with "
        "every reason that forbids it and the squares behind each reason.\n"
        "4. Call `facts` with the groups you need: `tactics` for one-ply threats, `king` for "
        "safety, `pawns` for structure, `threats` for what is hanging.\n"
        "5. Call `opening` for the ECO name where the position has one, and `ending` once the "
        "pieces are nearly gone: it names the ending, what theory says the result is, and the "
        "technique that gets it.\n\n"
        "Report what is true and name the squares it was read off. These tools do not search: a "
        "numeric evaluation or a best move needs an engine, which this server is not."
    )


def run() -> None:
    """Serves the tools over stdio."""
    mcp.run()
