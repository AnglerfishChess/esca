"""The compiled extension. Its public names are re-exported by `esca`."""

import os
from collections.abc import Iterable, Iterator, Sequence
from typing import Any, Final, Literal, TypedDict, final, overload

__version__: Final[str]

CLASSIC: Final[Variant]
CHESS960: Final[Variant]

KING_TO_ROOK: Final[CastlingStyle]
KING_TWO_SQUARES: Final[CastlingStyle]

POLYGLOT_ENTRY_SIZE: Final[int]

#: The name of a variant.
type VariantName = Literal["chess", "chess960"]

#: A piece: its role letter, upper case for White.
type PieceLetter = Literal["P", "N", "B", "R", "Q", "K", "p", "n", "b", "r", "q", "k"]

#: What kind of move this is, by the first case that applies.
type MoveKind = Literal["quiet", "capture", "en_passant", "castling", "promotion"]

#: How castling is spelled in UCI text.
type CastlingStyle = Literal["king_to_rook", "king_two_squares"]

#: Which castling, named by the file the king lands on: g short, c long.
type Wing = Literal["short", "long"]

#: How a game ended, with no claim from either player.
type Outcome = Literal[
    "checkmate",
    "stalemate",
    "insufficient_material",
    "seventy_five_moves",
    "fivefold_repetition",
]

#: A draw a player may claim, the position still playable until one does.
type DrawClaim = Literal["fifty_moves", "threefold_repetition"]

#: A draw that needs no claim.
type AutomaticDrawKind = Literal["stalemate", "insufficient_material", "fivefold_repetition", "seventy_five_moves"]

#: The material a variant calls insufficient, named.
type MaterialConfig = Literal["k_v_k", "kn_v_k", "kb_v_k", "kb_v_kb_same_colour"]

#: What keeps an en-passant capture off the board.
type EpObstacleKind = Literal["pinned", "exposes_king", "in_check"]

#: What holds a unit that has no legal move.
type StuckKind = Literal["pinned", "blocked", "no_moves"]

#: What resets the halfmove clock.
type ResetKind = Literal["capture", "pawn_move"]

#: What can tell two positions of one placement apart.
type Difference = Literal["castling_rights", "en_passant", "side_to_move"]

#: What theory says the result of an ending is.
type EndingResult = Literal["win", "usually_win", "usually_draw", "draw", "unknown"]

class Error(Exception):
    """Every error esca raises. Its members are `ValueError`s too."""

class FenError(Error, ValueError):
    """The text is not a FEN this crate can read."""

class IllegalMove(Error, ValueError):
    """The move is not legal in the position."""

class MoveParseError(Error, ValueError):
    """The text does not name one legal move of the position."""

class PositionError(Error, ValueError):
    """The position is not one this variant can play on."""

class ParseError(Error, ValueError):
    """The text does not name a square, file, rank, role or colour."""

class FieldEntry(TypedDict):
    """What one fact is called, what it holds and what it means."""

    name: str
    type: str
    meaning: str

class GroupEntry(TypedDict):
    """One group of facts, and the fields it holds."""

    name: str
    meaning: str
    fields: list[FieldEntry]

@final
class Variant:
    """One set of chess rules."""

    @staticmethod
    def named(name: VariantName) -> Variant: ...
    @property
    def name(self) -> VariantName: ...
    @property
    def promotion_roles(self) -> list[RoleLetter]: ...
    def start_position(self, seed: int = 0) -> Position: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class SquareSet:
    """A set of squares, named `e4`-style."""

    def __new__(cls, squares: Sequence[Square] | None = None) -> SquareSet: ...
    @property
    def bits(self) -> int: ...
    @property
    def squares(self) -> list[Square]: ...
    @property
    def files(self) -> FileSet: ...
    def is_subset(self, other: SquareSet) -> bool: ...
    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...
    def __contains__(self, square: str) -> bool: ...
    def __iter__(self) -> Iterator[Square]: ...
    def __and__(self, other: SquareSet) -> SquareSet: ...
    def __or__(self, other: SquareSet) -> SquareSet: ...
    def __xor__(self, other: SquareSet) -> SquareSet: ...
    def __sub__(self, other: SquareSet) -> SquareSet: ...
    def __invert__(self) -> SquareSet: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Move:
    """One action: origin, destination, promotion role and kind.

    Castling is stored king-to-rook, so the destination is the rook's own
    square. Two moves are equal when their origins, destinations and
    promotions are.
    """

    def __new__(
        cls,
        origin: Square,
        destination: Square,
        promotion: RoleName | None = None,
        kind: MoveKind = "quiet",
        is_capture: bool = False,
    ) -> Move: ...
    @staticmethod
    def from_uci(text: str) -> Move: ...
    @property
    def origin(self) -> Square: ...
    @property
    def destination(self) -> Square: ...
    @property
    def promotion(self) -> RoleLetter | None: ...
    @property
    def kind(self) -> MoveKind: ...
    @property
    def is_capture(self) -> bool: ...
    @property
    def is_castling(self) -> bool: ...
    @property
    def is_en_passant(self) -> bool: ...
    def uci(self, *, castling: CastlingStyle = "king_to_rook") -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Position:
    """Placement and state, with no rules attached.

    A question that needs rules takes the variant that answers it; a question
    that needs the moves played before it belongs to a `Game`.
    """

    @staticmethod
    def from_fen(fen: str) -> Position: ...
    @property
    def fen(self) -> str: ...
    @property
    def epd(self) -> str: ...
    @property
    def side_to_move(self) -> Colour: ...
    @property
    def castling_rights(self) -> str: ...
    @property
    def en_passant(self) -> Square | None: ...
    @property
    def halfmove_clock(self) -> int: ...
    @property
    def fullmove_number(self) -> int: ...
    @property
    def clocks_known(self) -> bool: ...
    @property
    def in_check(self) -> bool: ...
    @property
    def occupied(self) -> SquareSet: ...
    @property
    def key(self) -> int: ...
    @property
    def polyglot_key(self) -> int: ...
    def piece_at(self, square: Square) -> PieceLetter | None: ...
    def by_role(self, role: RoleName) -> SquareSet: ...
    def by_colour(self, colour: ColourName) -> SquareSet: ...
    def by_piece(self, role: RoleName, colour: ColourName) -> SquareSet: ...
    def king_of(self, colour: ColourName) -> Square: ...
    def see(self, square: Square) -> int: ...
    def see_capture(self, mv: Move) -> int: ...
    def castling(self, colour: ColourName, wing: Wing) -> Castling: ...
    def en_passant_status(self) -> EnPassant: ...
    def checkers(self) -> SquareSet: ...
    def attackers(self, square: Square, colour: ColourName) -> SquareSet: ...
    def between(self, origin: Square, destination: Square) -> SquareSet: ...
    def pins(self, colour: ColourName) -> list[Pin]: ...
    def skewers(self, colour: ColourName) -> list[Skewer]: ...
    def ending(self) -> Ending: ...
    def facts(self, *, variant: Variant | None = None) -> Facts: ...
    def legal_moves(self, *, variant: Variant | None = None) -> list[Move]: ...
    def annotated_moves(self, *, variant: Variant | None = None) -> list[AnnotatedMove]: ...
    def san(self, mv: Move, *, variant: Variant | None = None) -> str: ...
    def parse_san(self, text: str, *, variant: Variant | None = None) -> Move: ...
    def uci(
        self,
        mv: Move,
        *,
        variant: Variant | None = None,
        castling: CastlingStyle = "king_to_rook",
    ) -> str: ...
    def parse_uci(self, text: str, *, variant: Variant | None = None) -> Move: ...
    def play(self, mv: Move | str, *, variant: Variant | None = None) -> Position: ...
    def outcome(self, *, variant: Variant | None = None) -> Outcome | None: ...
    def mirrored(self) -> Position: ...
    def summary(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Game:
    """A variant, a start position and the moves played from it."""

    def __new__(cls, *, variant: Variant | None = None, seed: int = 0) -> Game: ...
    @staticmethod
    def from_fen(fen: str, *, variant: Variant | None = None) -> Game: ...
    @staticmethod
    def from_position(position: Position, *, variant: Variant | None = None) -> Game: ...
    @property
    def variant(self) -> Variant: ...
    @property
    def position(self) -> Position: ...
    @property
    def start_position(self) -> Position: ...
    @property
    def moves(self) -> list[Move]: ...
    @property
    def positions(self) -> list[Position]: ...
    @property
    def ply(self) -> int: ...
    @property
    def castling_output(self) -> CastlingStyle: ...
    @castling_output.setter
    def castling_output(self, style: CastlingStyle) -> None: ...
    def copy(self) -> Game: ...
    def legal_moves(self) -> list[Move]: ...
    def annotated_moves(self) -> list[AnnotatedMove]: ...
    def play(self, mv: Move | str) -> None: ...
    def play_san(self, text: str) -> None: ...
    def undo(self) -> Move | None: ...
    def move_to_uci(self, mv: Move) -> str: ...
    def move_to_san(self, mv: Move) -> str: ...
    def outcome(self) -> Outcome | None: ...
    def claims(self) -> list[DrawClaim]: ...
    def repetitions(self) -> int: ...
    def repetition_status(self) -> Repetition: ...
    def fifty_move_status(self) -> FiftyMove: ...
    def draw_status(self) -> DrawStatus: ...
    def claims_after(self, mv: Move) -> list[ClaimableDraw]: ...
    def ending(self) -> Ending: ...
    def facts(self) -> Facts: ...
    def opening(self) -> Opening | None: ...
    def to_pgn(self) -> PgnGame: ...

@final
class Castling:
    """One castling of one colour, and everything standing in its way."""

    @property
    def colour(self) -> Colour: ...
    @property
    def wing(self) -> Wing: ...
    @property
    def right(self) -> bool: ...
    @property
    def rook_present(self) -> bool: ...
    @property
    def king_in_check_by(self) -> SquareSet: ...
    @property
    def path_attacked(self) -> list[tuple[Square, SquareSet]]: ...
    @property
    def path_blocked(self) -> SquareSet: ...
    @property
    def allowed(self) -> bool: ...
    def describe(self) -> str: ...

@final
class EnPassant:
    """The en-passant capture a position offers the side to move."""

    @property
    def target(self) -> Square | None: ...
    @property
    def captures(self) -> list[EpCapture]: ...
    def describe(self) -> str: ...

@final
class EpCapture:
    """One pawn's en-passant capture of the target."""

    @property
    def origin(self) -> Square: ...
    @property
    def legal(self) -> bool: ...
    @property
    def forbidden_by(self) -> EpObstacle | None: ...
    def describe(self) -> str: ...

@final
class EpObstacle:
    """What keeps an en-passant capture off the board."""

    @property
    def kind(self) -> EpObstacleKind: ...
    @property
    def ray(self) -> SquareSet: ...
    @property
    def pinner(self) -> Square | None: ...
    @property
    def attacker(self) -> Square | None: ...
    @property
    def by(self) -> SquareSet: ...
    def describe(self) -> str: ...

@final
class Pin:
    """A unit that may not move off the line between a slider and its king."""

    @property
    def pinned(self) -> Square: ...
    @property
    def pinner(self) -> Square: ...
    @property
    def king(self) -> Square: ...
    @property
    def ray(self) -> SquareSet: ...
    def describe(self) -> str: ...

@final
class Skewer:
    """A unit attacked with a less valuable one directly behind it."""

    @property
    def attacker(self) -> Square: ...
    @property
    def front(self) -> Square: ...
    @property
    def behind(self) -> Square: ...
    @property
    def ray(self) -> SquareSet: ...
    def describe(self) -> str: ...

@final
class Repetition:
    """How often the current position has stood, and what nearly counted."""

    @property
    def count(self) -> int: ...
    @property
    def plies(self) -> list[int]: ...
    @property
    def near_misses(self) -> list[NearMiss]: ...
    def describe(self) -> str: ...

@final
class NearMiss:
    """An earlier ply with the same placement that is not a repetition."""

    @property
    def ply(self) -> int: ...
    @property
    def differs(self) -> list[Difference]: ...
    def describe(self) -> str: ...

@final
class FiftyMove:
    """The halfmove clock, and how far it is from ending the game."""

    @property
    def clock(self) -> int: ...
    @property
    def plies_to_claim(self) -> int: ...
    @property
    def plies_to_automatic(self) -> int: ...
    @property
    def last_reset(self) -> Reset | None: ...
    def describe(self) -> str: ...

@final
class Reset:
    """The move that last set the halfmove clock to 0."""

    @property
    def ply(self) -> int: ...
    @property
    def kind(self) -> ResetKind: ...
    def describe(self) -> str: ...

@final
class DrawStatus:
    """Every draw condition that holds, not the first of them."""

    @property
    def automatic(self) -> list[AutomaticDraw]: ...
    @property
    def claimable(self) -> list[ClaimableDraw]: ...
    def describe(self) -> str: ...

@final
class AutomaticDraw:
    """A draw that needs no claim."""

    @property
    def kind(self) -> AutomaticDrawKind: ...
    @property
    def stalemate(self) -> StalemateDetail | None: ...
    @property
    def material(self) -> MaterialConfig | None: ...
    @property
    def repetition(self) -> Repetition | None: ...
    @property
    def fifty_move(self) -> FiftyMove | None: ...
    def describe(self) -> str: ...

@final
class ClaimableDraw:
    """A draw a player may ask for."""

    @property
    def kind(self) -> DrawClaim: ...
    @property
    def repetition(self) -> Repetition | None: ...
    @property
    def fifty_move(self) -> FiftyMove | None: ...
    def describe(self) -> str: ...

@final
class StalemateDetail:
    """Why the side to move has no move."""

    @property
    def king(self) -> Square: ...
    @property
    def escape_squares(self) -> list[tuple[Square, SquareSet]]: ...
    @property
    def stuck_units(self) -> list[tuple[Square, Stuck]]: ...
    def describe(self) -> str: ...

@final
class Stuck:
    """What holds a unit that has no legal move."""

    @property
    def kind(self) -> StuckKind: ...
    @property
    def ray(self) -> SquareSet: ...
    @property
    def pinner(self) -> Square | None: ...
    def describe(self) -> str: ...

@final
class EndingClass:
    """One named ending of the catalogue, equal to and printing as its name."""

    @staticmethod
    def all() -> list[EndingClass]: ...
    @property
    def name(self) -> str: ...
    def describe(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class EndingVerdict:
    """What theory says the result of an ending is."""

    @property
    def name(self) -> EndingResult: ...
    @property
    def winner(self) -> Colour | None: ...
    def describe(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class EndingTechnique:
    """The named method an ending is played by."""

    @staticmethod
    def all() -> list[EndingTechnique]: ...
    @property
    def name(self) -> str: ...
    def describe(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class MaterialSignature:
    """The material both sides hold, written the way endings are named."""

    @property
    def text(self) -> str: ...
    @property
    def stronger(self) -> Colour: ...
    @property
    def pawns(self) -> int: ...
    def count(self, colour: ColourName, role: RoleName) -> int: ...
    def pieces(self, colour: ColourName) -> int: ...
    def value(self, colour: ColourName) -> int: ...
    def describe(self) -> str: ...

@final
class PawnRace:
    """The race of the only pawn on the board."""

    @property
    def pawn(self) -> Square: ...
    @property
    def colour(self) -> Colour: ...
    @property
    def promotion(self) -> Square: ...
    @property
    def rook_pawn(self) -> bool: ...
    @property
    def steps(self) -> int: ...
    @property
    def defender_inside_square(self) -> bool: ...
    @property
    def attacker_in_front(self) -> bool: ...
    @property
    def defender_in_front(self) -> bool: ...
    @property
    def defender_holds_the_corner(self) -> bool: ...
    def describe(self) -> str: ...

@final
class Bishops:
    """The bishops on the board, when at least one stands on it."""

    @property
    def opposite_colours(self) -> bool: ...
    @property
    def same_colour(self) -> bool: ...
    @property
    def wrong_bishop(self) -> bool: ...
    def describe(self) -> str: ...

@final
class EndingEvidence:
    """The position-specific facts an ending's verdict and technique are read off."""

    @property
    def pawn(self) -> PawnRace | None: ...
    @property
    def bishops(self) -> Bishops | None: ...
    @property
    def opposition(self) -> bool: ...
    def describe(self) -> str: ...

@final
class Ending:
    """What ending a position is, and what is known about it."""

    @property
    def class_(self) -> EndingClass: ...
    @property
    def signature(self) -> MaterialSignature: ...
    @property
    def verdict(self) -> EndingVerdict: ...
    @property
    def technique(self) -> EndingTechnique: ...
    @property
    def evidence(self) -> EndingEvidence: ...
    def describe(self) -> str: ...

@final
class FileSet:
    """A set of files."""

    def __new__(cls, files: Sequence[FileName] | None = None) -> FileSet: ...
    @property
    def files(self) -> list[FileLetter]: ...
    def is_subset(self, other: FileSet) -> bool: ...
    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...
    def __contains__(self, file: str) -> bool: ...
    def __iter__(self) -> Iterator[FileLetter]: ...
    def __and__(self, other: FileSet) -> FileSet: ...
    def __or__(self, other: FileSet) -> FileSet: ...
    def __sub__(self, other: FileSet) -> FileSet: ...
    def __invert__(self) -> FileSet: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

# --- generated from the facts catalogue: do not edit by hand ---
#
# cargo run --example catalogue --all-features

#: A colour, as every esca surface writes one.
type Colour = Literal["white", "black"]

#: A colour, as every esca surface reads one: either spelling, either case.
type ColourName = Literal["white", "black", "White", "Black", "WHITE", "BLACK", "w", "b", "W", "B"]

#: A file letter.
type FileLetter = Literal["a", "b", "c", "d", "e", "f", "g", "h"]

#: A file, as a file argument reads one: either case.
type FileName = Literal[
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
]

#: A role letter, as every esca surface spells a piece kind.
type RoleLetter = Literal["p", "n", "b", "r", "q", "k"]

#: A role, as a role argument reads one: either case.
type RoleName = Literal["p", "n", "b", "r", "q", "k", "P", "N", "B", "R", "Q", "K"]

#: A square name, a1 through h8.
type Square = Literal[
    "a1",
    "b1",
    "c1",
    "d1",
    "e1",
    "f1",
    "g1",
    "h1",
    "a2",
    "b2",
    "c2",
    "d2",
    "e2",
    "f2",
    "g2",
    "h2",
    "a3",
    "b3",
    "c3",
    "d3",
    "e3",
    "f3",
    "g3",
    "h3",
    "a4",
    "b4",
    "c4",
    "d4",
    "e4",
    "f4",
    "g4",
    "h4",
    "a5",
    "b5",
    "c5",
    "d5",
    "e5",
    "f5",
    "g5",
    "h5",
    "a6",
    "b6",
    "c6",
    "d6",
    "e6",
    "f6",
    "g6",
    "h6",
    "a7",
    "b7",
    "c7",
    "d7",
    "e7",
    "f7",
    "g7",
    "h7",
    "a8",
    "b8",
    "c8",
    "d8",
    "e8",
    "f8",
    "g8",
    "h8",
]

#: The side a king's square and its spent castling rights read as.
type CastledSide = Literal["short", "long"]

#: Which opposition the kings stand in.
type Opposition = Literal["direct", "distant"]

#: A material configuration that draws although one side is ahead.
type DrawishMaterial = Literal["two_knights", "wrong_bishop", "opposite_bishops"]

#: Eight values, file a first, through file h.
type PerFile[T] = tuple[T, T, T, T, T, T, T, T]

#: Eight values, rank 1 first, through rank 8.
type PerRank[T] = tuple[T, T, T, T, T, T, T, T]

class ColourPair[T](TypedDict):
    """One value per colour, as `to_dict()` writes a `ByColour`."""

    white: T
    black: T

@final
class ByColour[T]:
    """One value for White and one for Black.

    Iterating gives the two values, White first, so `white, black = pair`
    unpacks; `items()` gives each with the colour it belongs to. A pair hashes
    when both of its values do.
    """

    __match_args__: Final[tuple[Literal["white"], Literal["black"]]]

    def __new__(cls, white: T, black: T) -> ByColour[T]: ...
    @property
    def white(self) -> T: ...
    @property
    def black(self) -> T: ...
    def of(self, colour: ColourName) -> T: ...
    def items(self) -> list[tuple[Colour, T]]: ...
    def __getitem__(self, colour: ColourName) -> T: ...
    def __iter__(self) -> Iterator[T]: ...
    def __len__(self) -> int: ...
    def __contains__(self, colour: str) -> bool: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __class_getitem__(cls, item: Any) -> Any: ...

class PlacementFactsDict(TypedDict):
    """The plain data `PlacementFacts.to_dict()` answers."""

    units: ColourPair[list[str]]
    pawns: ColourPair[list[str]]
    knights: ColourPair[list[str]]
    bishops: ColourPair[list[str]]
    rooks: ColourPair[list[str]]
    queens: ColourPair[list[str]]
    kings: ColourPair[list[str]]

class StateFactsDict(TypedDict):
    """The plain data `StateFacts.to_dict()` answers."""

    in_check: bool
    double_check: bool
    can_castle_short: ColourPair[bool]
    can_castle_long: ColourPair[bool]
    en_passant: FileLetter | None
    en_passant_capture_legal: bool

class HistoryFactsDict(TypedDict):
    """The plain data `HistoryFacts.to_dict()` answers."""

    known: bool
    halfmove_clock: int
    halfmove_known: bool
    repetition_seen: bool
    repetition_available: bool
    captures_in_last_8: int
    checks_in_last_8: int
    quiet_plies: int
    material_trend: int
    last_move_victim: RoleLetter | None
    last_move_mover: RoleLetter | None

class MaterialFactsDict(TypedDict):
    """The plain data `MaterialFacts.to_dict()` answers."""

    pawns: ColourPair[int]
    knights: ColourPair[int]
    bishops: ColourPair[int]
    rooks: ColourPair[int]
    queens: ColourPair[int]
    non_pawn_value: ColourPair[int]
    value: ColourPair[int]
    phase_points: int
    both_queens: bool
    pawns_only: bool
    insufficient_material: ColourPair[bool]

class PawnFactsDict(TypedDict):
    """The plain data `PawnFacts.to_dict()` answers."""

    pawns: ColourPair[list[str]]
    passed: ColourPair[list[str]]
    candidates: ColourPair[list[str]]
    doubled: ColourPair[list[str]]
    isolated: ColourPair[list[str]]
    backward: ColourPair[list[str]]
    defended: ColourPair[list[str]]
    count_by_file: ColourPair[list[int]]
    count_by_rank: ColourPair[list[int]]
    open_files: list[str]
    semi_open_files: ColourPair[list[str]]
    islands: ColourPair[int]
    levers: ColourPair[int]
    rams: int
    passer_lead_rank: ColourPair[int | None]
    passer_protected: ColourPair[int]
    passers_connected: ColourPair[bool]
    passer_unstoppable: ColourPair[bool]
    chain_max_length: ColourPair[int]
    chain_base_attacked: ColourPair[bool]
    queenside_majority: ColourPair[bool]
    kingside_majority: ColourPair[bool]
    holes: ColourPair[list[str]]
    holes_occupied: ColourPair[int]
    fixed_pawns: ColourPair[int]
    blocked_passers: ColourPair[int]
    passer_distance: ColourPair[int | None]
    passer_own_king_distance: ColourPair[int | None]
    passer_enemy_king_distance: ColourPair[int | None]
    passer_in_square: ColourPair[bool]
    passer_free_path: ColourPair[int]
    half_open_at_enemy_king: ColourPair[int]
    backward_on_semi_open: ColourPair[int]

class PieceFactsDict(TypedDict):
    """The plain data `PieceFacts.to_dict()` answers."""

    bishop_pair: ColourPair[bool]
    knight_pair: ColourPair[bool]
    bishops_light: ColourPair[int]
    bishops_dark: ColourPair[int]
    opposite_coloured_bishops: bool
    pawns_on_bishop_colour: ColourPair[int]
    rooks_connected_rank: ColourPair[bool]
    rooks_connected_file: ColourPair[bool]
    rooks_on_open_file: ColourPair[int]
    rooks_on_semi_open_file: ColourPair[int]
    rooks_on_seventh_rank: ColourPair[int]
    rook_behind_own_passer: ColourPair[int]
    rook_behind_enemy_passer: ColourPair[int]
    trapped_rook: ColourPair[bool]
    outposts: ColourPair[list[str]]
    minors_on_outpost: ColourPair[int]
    outpost_squares_free: ColourPair[int]
    knights_on_rim: ColourPair[int]
    minors_undeveloped: ColourPair[int]
    queen_developed: ColourPair[bool]
    fixed_pawns_on_bishop_colour: ColourPair[int]
    rook_on_seventh_with_king_on_eighth: ColourPair[bool]
    trapped_pieces: ColourPair[int]
    trapped_value: ColourPair[int]

class KingFactsDict(TypedDict):
    """The plain data `KingFacts.to_dict()` answers."""

    square: ColourPair[str]
    on_home_square: ColourPair[bool]
    castled_queenside: ColourPair[bool]
    castled_kingside: ColourPair[bool]
    shield_files: ColourPair[list[FileLetter]]
    shield: ColourPair[list[int | None]]
    shield_file_open: ColourPair[list[bool]]
    shield_file_semi_open_for_enemy: ColourPair[list[bool]]
    storm: ColourPair[list[int | None]]
    ring: ColourPair[list[str]]
    ring_attackers: ColourPair[int]
    ring_attack_weight: ColourPair[int]
    ring_defended: ColourPair[int]
    ring_holes: ColourPair[int]
    escape_squares: ColourPair[int]
    back_rank_risk: ColourPair[bool]
    distance: int
    tropism: ColourPair[float]
    virtual_mobility: ColourPair[int]
    ring_defenders: ColourPair[int]
    ring_defence_weight: ColourPair[int]
    open_rays: ColourPair[int]
    luft: ColourPair[bool]
    castled_side: ColourPair[CastledSide | None]
    opposite_side_castling: bool

class MobilityFactsDict(TypedDict):
    """The plain data `MobilityFacts.to_dict()` answers."""

    pawns: ColourPair[int]
    knights: ColourPair[int]
    bishops: ColourPair[int]
    rooks: ColourPair[int]
    queens: ColourPair[int]
    safe_pawns: ColourPair[int]
    safe_knights: ColourPair[int]
    safe_bishops: ColourPair[int]
    safe_rooks: ColourPair[int]
    safe_queens: ColourPair[int]
    total: ColourPair[int]
    space: ColourPair[int]
    centre_control: ColourPair[int]
    extended_centre_control: ColourPair[int]
    immobile_pieces: ColourPair[int]

class AttackFactsDict(TypedDict):
    """The plain data `AttackFacts.to_dict()` answers."""

    controlled: ColourPair[list[str]]
    by_pawns: ColourPair[list[str]]
    by_knights: ColourPair[list[str]]
    by_bishops: ColourPair[list[str]]
    by_rooks: ColourPair[list[str]]
    by_queens: ColourPair[list[str]]
    by_kings: ColourPair[list[str]]
    attacked: ColourPair[list[str]]
    hanging: ColourPair[list[str]]
    en_prise: ColourPair[list[str]]
    pinned: ColourPair[list[str]]
    defended: ColourPair[list[str]]
    attacked_value: ColourPair[int]
    hanging_value: ColourPair[int]
    en_prise_value: ColourPair[int]
    en_prise_max_value: ColourPair[int]
    pinned_value: ColourPair[int]
    skewer_candidates: ColourPair[int]

class ExchangeFactsDict(TypedDict):
    """The plain data `ExchangeFacts.to_dict()` answers."""

    see_best_capture: ColourPair[int]
    see_positive_capture_count: ColourPair[int]
    see_equal_capture_count: ColourPair[int]
    see_positive_total: ColourPair[int]

class ThreatFactsDict(TypedDict):
    """The plain data `ThreatFacts.to_dict()` answers."""

    threatened: ColourPair[list[str]]
    threatened_value: ColourPair[int]
    threat_max_gain: ColourPair[int]
    attacked_by_lesser: ColourPair[list[str]]
    queen_attacked_by_lesser: ColourPair[bool]
    overloaded_defenders: ColourPair[list[str]]
    removable_defenders: ColourPair[list[str]]
    loose: ColourPair[list[str]]
    attacker_surplus: ColourPair[list[str]]
    xray_through_enemy: ColourPair[int]
    battery_count: ColourPair[int]
    battery_at_king: ColourPair[bool]

class TacticsFactsDict(TypedDict):
    """The plain data `TacticsFacts.to_dict()` answers."""

    available: ColourPair[bool]
    check_count: ColourPair[int]
    check_by_pawn: ColourPair[bool]
    check_by_knight: ColourPair[bool]
    check_by_bishop: ColourPair[bool]
    check_by_rook: ColourPair[bool]
    check_by_queen: ColourPair[bool]
    safe_check_count: ColourPair[int]
    safe_check_by_pawn: ColourPair[bool]
    safe_check_by_knight: ColourPair[bool]
    safe_check_by_bishop: ColourPair[bool]
    safe_check_by_rook: ColourPair[bool]
    safe_check_by_queen: ColourPair[bool]
    double_check_available: ColourPair[bool]
    discovered_check_available: ColourPair[bool]
    mate_in_1: ColourPair[bool]
    stalemate_in_1: ColourPair[bool]
    promotion_files: ColourPair[list[str]]
    promotes_to_queen: ColourPair[bool]
    promotes_to_rook: ColourPair[bool]
    promotes_to_bishop: ColourPair[bool]
    promotes_to_knight: ColourPair[bool]
    safe_promotion_files: ColourPair[list[str]]
    capture_count: ColourPair[int]
    winning_capture_available: ColourPair[bool]
    winning_capture_max_gain: ColourPair[int]
    captures_hanging: ColourPair[bool]
    hanging_victim_max_value: ColourPair[int]
    equal_capture_count: ColourPair[int]
    losing_capture_count: ColourPair[int]
    fork_count: ColourPair[int]
    fork_max_value: ColourPair[int]
    knight_fork_available: ColourPair[bool]
    royal_fork_available: ColourPair[bool]
    pin_creation_count: ColourPair[int]
    skewer_creation_available: ColourPair[bool]
    discovered_attack_available: ColourPair[bool]
    legal_move_count: ColourPair[int]
    safe_check_capturing: ColourPair[bool]
    discovered_attack_on_queen: ColourPair[bool]
    back_rank_mate_threat: ColourPair[bool]
    quiet_threat_available: ColourPair[bool]
    no_safe_moves: ColourPair[bool]
    promotion_see_positive: ColourPair[bool]

class EndgameFactsDict(TypedDict):
    """The plain data `EndgameFacts.to_dict()` answers."""

    king_centralisation: ColourPair[int]
    race_plies: ColourPair[int]
    opposition: Opposition | None
    key_square_occupied: ColourPair[bool]
    wrong_colour_bishop: ColourPair[bool]
    drawish_material: DrawishMaterial | None

class MapFactsDict(TypedDict):
    """The plain data `MapFacts.to_dict()` answers."""

    controlled: ColourPair[list[str]]
    by_pawns: ColourPair[list[str]]
    hanging: ColourPair[list[str]]
    pinned: ColourPair[list[str]]
    threatened: ColourPair[list[str]]

class MoveFactsDict(TypedDict):
    """The plain data `MoveFacts.to_dict()` answers."""

    victim: RoleLetter | None
    mover: RoleLetter
    promotion: RoleLetter | None
    gives_check: bool
    gives_safe_check: bool
    is_safe: bool
    captures_hanging: bool
    escapes_attack: bool
    to_attacked_by_pawn: bool
    is_castling: bool
    is_en_passant: bool
    see: int
    threat_created_max: int
    moves_attacked_unit: bool
    blocks_check: bool
    advances_passer: bool
    creates_passer: bool
    creates_isolated: bool
    creates_doubled: bool
    creates_backward: bool
    opens_file_at_enemy_king: bool
    enemy_king_attackers_delta: int
    mover_king_attackers_delta: int
    mover_hanging_delta: int
    enemy_hanging_delta: int
    leaves_unit_hanging: bool
    gives_discovered_attack: bool

class FactsDict(TypedDict):
    """The plain data `Facts.to_dict()` answers, one entry per group."""

    placement: PlacementFactsDict
    state: StateFactsDict
    history: HistoryFactsDict
    material: MaterialFactsDict
    pawns: PawnFactsDict
    pieces: PieceFactsDict
    king: KingFactsDict
    mobility: MobilityFactsDict
    attacks: AttackFactsDict
    exchange: ExchangeFactsDict
    threats: ThreatFactsDict
    tactics: TacticsFactsDict
    endgame: EndgameFactsDict
    maps: MapFactsDict

@final
class PlacementFacts:
    """Where the units stand."""

    @property
    def units(self) -> ByColour[SquareSet]:
        """Every unit of the side."""
    @property
    def pawns(self) -> ByColour[SquareSet]:
        """The side's pawns."""
    @property
    def knights(self) -> ByColour[SquareSet]:
        """The side's knights."""
    @property
    def bishops(self) -> ByColour[SquareSet]:
        """The side's bishops."""
    @property
    def rooks(self) -> ByColour[SquareSet]:
        """The side's rooks."""
    @property
    def queens(self) -> ByColour[SquareSet]:
        """The side's queens."""
    @property
    def kings(self) -> ByColour[SquareSet]:
        """The side's king."""
    def to_dict(self) -> PlacementFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class StateFacts:
    """Check, castling rights and the en-passant square."""

    @property
    def in_check(self) -> bool:
        """The side to move stands in check."""
    @property
    def double_check(self) -> bool:
        """Two or more units give check to the side to move."""
    @property
    def can_castle_short(self) -> ByColour[bool]:
        """The side may still castle short."""
    @property
    def can_castle_long(self) -> ByColour[bool]:
        """The side may still castle long."""
    @property
    def en_passant(self) -> FileLetter | None:
        """The file the position names as the en-passant target, if any."""
    @property
    def en_passant_capture_legal(self) -> bool:
        """Some legal move captures en passant."""
    def to_dict(self) -> StateFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class HistoryFacts:
    """What the plies before this position say: the clock, repetition, and how forcing the recent play was.
    Everything but the halfmove clock, which the position itself carries, is zero unless `known` is true.
    """

    @property
    def known(self) -> bool:
        """A position history was supplied."""
    @property
    def halfmove_clock(self) -> int:
        """Plies since the last capture or pawn move."""
    @property
    def halfmove_known(self) -> bool:
        """The position carried a halfmove clock."""
    @property
    def repetition_seen(self) -> bool:
        """This position occurred before in the supplied history."""
    @property
    def repetition_available(self) -> bool:
        """Some legal move reaches a position of the supplied history."""
    @property
    def captures_in_last_8(self) -> int:
        """Captures among the last eight plies of the supplied history."""
    @property
    def checks_in_last_8(self) -> int:
        """Plies among the last eight that gave check."""
    @property
    def quiet_plies(self) -> int:
        """Plies since the last one that captured or gave check; the whole history when it holds none."""
    @property
    def material_trend(self) -> int:
        """The material balance now, less the balance eight plies ago, White less Black."""
    @property
    def last_move_victim(self) -> RoleLetter | None:
        """The role the last move captured, the pawn for an en-passant capture."""
    @property
    def last_move_mover(self) -> RoleLetter | None:
        """The role that made the last move; for castling, the king."""
    def to_dict(self) -> HistoryFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class MaterialFacts:
    """Material and phase."""

    @property
    def pawns(self) -> ByColour[int]:
        """The side's pawns."""
    @property
    def knights(self) -> ByColour[int]:
        """The side's knights."""
    @property
    def bishops(self) -> ByColour[int]:
        """The side's bishops."""
    @property
    def rooks(self) -> ByColour[int]:
        """The side's rooks."""
    @property
    def queens(self) -> ByColour[int]:
        """The side's queens."""
    @property
    def non_pawn_value(self) -> ByColour[int]:
        """The value of the side's knights, bishops, rooks and queens."""
    @property
    def value(self) -> ByColour[int]:
        """The value of every unit of the side but the king."""
    @property
    def phase_points(self) -> int:
        """`4·Q + 2·R + B + N` over both sides: 24 at the full opening array, 0 with nothing but kings and
        pawns left.
        """
    @property
    def both_queens(self) -> bool:
        """Both sides have at least one queen."""
    @property
    def pawns_only(self) -> bool:
        """Neither side has a unit other than kings and pawns."""
    @property
    def insufficient_material(self) -> ByColour[bool]:
        """The side's own material could never deliver mate."""
    def to_dict(self) -> MaterialFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class PawnFacts:
    """Pawn structure."""

    @property
    def pawns(self) -> ByColour[SquareSet]:
        """The side's pawns."""
    @property
    def passed(self) -> ByColour[SquareSet]:
        """The side's pawns with no enemy pawn ahead on their own or an adjacent file."""
    @property
    def candidates(self) -> ByColour[SquareSet]:
        """The side's candidate passers: pawns with no enemy pawn ahead on their own file, backed by at least as
        many friendly pawns behind on the adjacent files as there are enemy pawns ahead on them.
        """
    @property
    def doubled(self) -> ByColour[SquareSet]:
        """The side's pawns sharing a file with another pawn of their own colour."""
    @property
    def isolated(self) -> ByColour[SquareSet]:
        """The side's pawns with no friendly pawn on either adjacent file."""
    @property
    def backward(self) -> ByColour[SquareSet]:
        """The side's pawns that cannot advance without being taken and cannot be defended by a pawn."""
    @property
    def defended(self) -> ByColour[SquareSet]:
        """The side's pawns defended by a pawn of their own colour."""
    @property
    def count_by_file(self) -> ByColour[PerFile[int]]:
        """The side's pawns per file, file a first."""
    @property
    def count_by_rank(self) -> ByColour[PerRank[int]]:
        """The side's pawns per rank counted from its own back rank, rank 1 first."""
    @property
    def open_files(self) -> FileSet:
        """Files carrying no pawn of either colour."""
    @property
    def semi_open_files(self) -> ByColour[FileSet]:
        """Files carrying no pawn of the side and at least one of the other."""
    @property
    def islands(self) -> ByColour[int]:
        """The side's maximal runs of adjacent files carrying a pawn."""
    @property
    def levers(self) -> ByColour[int]:
        """The side's pawns that can capture an enemy pawn."""
    @property
    def rams(self) -> int:
        """Pawn pairs blocking each other head on."""
    @property
    def passer_lead_rank(self) -> ByColour[int | None]:
        """The rank, counted from the side's own back rank, of its most advanced passer."""
    @property
    def passer_protected(self) -> ByColour[int]:
        """The side's passers defended by a friendly pawn."""
    @property
    def passers_connected(self) -> ByColour[bool]:
        """The side has two passers on adjacent files."""
    @property
    def passer_unstoppable(self) -> ByColour[bool]:
        """The side has a passer the enemy king cannot catch."""
    @property
    def chain_max_length(self) -> ByColour[int]:
        """The side's longest run of pawns each defending the next."""
    @property
    def chain_base_attacked(self) -> ByColour[bool]:
        """An enemy unit attacks the base of one of the side's chains of two or more."""
    @property
    def queenside_majority(self) -> ByColour[bool]:
        """The side has more pawns than the enemy on files a to d."""
    @property
    def kingside_majority(self) -> ByColour[bool]:
        """The side has more pawns than the enemy on files e to h."""
    @property
    def holes(self) -> ByColour[SquareSet]:
        """Squares on the side's ranks 3 to 6 that no pawn of the side can ever attack."""
    @property
    def holes_occupied(self) -> ByColour[int]:
        """Enemy knights and bishops standing on those squares."""
    @property
    def fixed_pawns(self) -> ByColour[int]:
        """The side's pawns whose stop square holds a unit of either colour."""
    @property
    def blocked_passers(self) -> ByColour[int]:
        """The side's passers whose stop square holds an enemy unit."""
    @property
    def passer_distance(self) -> ByColour[int | None]:
        """The pushes the side's lead passer still needs."""
    @property
    def passer_own_king_distance(self) -> ByColour[int | None]:
        """The distance from the side's own king to its lead passer's promotion square."""
    @property
    def passer_enemy_king_distance(self) -> ByColour[int | None]:
        """The distance from the enemy king to that same square."""
    @property
    def passer_in_square(self) -> ByColour[bool]:
        """The enemy king stands in the square of the side's lead passer."""
    @property
    def passer_free_path(self) -> ByColour[int]:
        """The side's passers whose whole front span is empty."""
    @property
    def half_open_at_enemy_king(self) -> ByColour[int]:
        """Files semi-open for the side among the enemy king's three shield files."""
    @property
    def backward_on_semi_open(self) -> ByColour[int]:
        """The side's backward pawns on a file semi-open for the enemy."""
    def to_dict(self) -> PawnFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class PieceFacts:
    """Bishops, rooks, knights and queens."""

    @property
    def bishop_pair(self) -> ByColour[bool]:
        """The side holds bishops on both square colours."""
    @property
    def knight_pair(self) -> ByColour[bool]:
        """The side holds two or more knights."""
    @property
    def bishops_light(self) -> ByColour[int]:
        """The side's bishops on light squares."""
    @property
    def bishops_dark(self) -> ByColour[int]:
        """The side's bishops on dark squares."""
    @property
    def opposite_coloured_bishops(self) -> bool:
        """Exactly one bishop each, on different square colours."""
    @property
    def pawns_on_bishop_colour(self) -> ByColour[int]:
        """The side's pawns standing on a square colour of one of its own bishops."""
    @property
    def rooks_connected_rank(self) -> ByColour[bool]:
        """Two of the side's rooks share a rank with nothing between."""
    @property
    def rooks_connected_file(self) -> ByColour[bool]:
        """Two of the side's rooks share a file with nothing between."""
    @property
    def rooks_on_open_file(self) -> ByColour[int]:
        """The side's rooks on an open file."""
    @property
    def rooks_on_semi_open_file(self) -> ByColour[int]:
        """The side's rooks on a file semi-open for their own side."""
    @property
    def rooks_on_seventh_rank(self) -> ByColour[int]:
        """The side's rooks on its seventh rank, counted from its own back rank."""
    @property
    def rook_behind_own_passer(self) -> ByColour[int]:
        """The side's rooks behind a passer of their own side."""
    @property
    def rook_behind_enemy_passer(self) -> ByColour[int]:
        """The side's rooks behind an enemy passer."""
    @property
    def trapped_rook(self) -> ByColour[bool]:
        """The side has a rook shut in by its own king with no file to leave by."""
    @property
    def outposts(self) -> ByColour[SquareSet]:
        """The side's outpost squares: squares on its ranks 4 to 6 that a friendly pawn defends and no enemy pawn
        can ever attack.
        """
    @property
    def minors_on_outpost(self) -> ByColour[int]:
        """The side's knights and bishops standing on one of them."""
    @property
    def outpost_squares_free(self) -> ByColour[int]:
        """Its unoccupied outpost squares."""
    @property
    def knights_on_rim(self) -> ByColour[int]:
        """The side's knights on file a or h, or on its rank 1 or 8."""
    @property
    def minors_undeveloped(self) -> ByColour[int]:
        """The side's knights and bishops still on their starting squares."""
    @property
    def queen_developed(self) -> ByColour[bool]:
        """A queen of the side stands off its starting square."""
    @property
    def fixed_pawns_on_bishop_colour(self) -> ByColour[int]:
        """The side's fixed pawns standing on a square colour of one of its own bishops."""
    @property
    def rook_on_seventh_with_king_on_eighth(self) -> ByColour[bool]:
        """The side has a rook on its seventh rank with the enemy king on its eighth."""
    @property
    def trapped_pieces(self) -> ByColour[int]:
        """The side's units that are neither pawns nor kings and have no safe destination."""
    @property
    def trapped_value(self) -> ByColour[int]:
        """The value sum of those."""
    def to_dict(self) -> PieceFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class KingFacts:
    """King safety and shelter."""

    @property
    def square(self) -> ByColour[str]:
        """Where the side's king stands."""
    @property
    def on_home_square(self) -> ByColour[bool]:
        """It stands on its own starting square."""
    @property
    def castled_queenside(self) -> ByColour[bool]:
        """It stands on files a to c."""
    @property
    def castled_kingside(self) -> ByColour[bool]:
        """It stands on files f to h."""
    @property
    def shield_files(self) -> ByColour[tuple[FileLetter, FileLetter, FileLetter]]:
        """The three files the king's shelter is read on, queenside first."""
    @property
    def shield(self) -> ByColour[tuple[int | None, int | None, int | None]]:
        """Ranks to the nearest friendly pawn ahead of the king, one per file of the three around the king,
        queenside first; `shield_files` names them.
        """
    @property
    def shield_file_open(self) -> ByColour[tuple[bool, bool, bool]]:
        """Whether each of the three files around the king carries no pawn of either colour, queenside first;
        `shield_files` names them.
        """
    @property
    def shield_file_semi_open_for_enemy(self) -> ByColour[tuple[bool, bool, bool]]:
        """Whether each of those three files is semi-open for the enemy of that king, queenside first;
        `shield_files` names them.
        """
    @property
    def storm(self) -> ByColour[tuple[int | None, int | None, int | None]]:
        """Ranks to the nearest enemy pawn ahead of the king, one per file of the three around the king,
        queenside first; `shield_files` names them.
        """
    @property
    def ring(self) -> ByColour[SquareSet]:
        """The squares adjacent to the king."""
    @property
    def ring_attackers(self) -> ByColour[int]:
        """Enemy knights, bishops, rooks and queens attacking that ring."""
    @property
    def ring_attack_weight(self) -> ByColour[int]:
        """Σ over those attackers of N, B = 1, R = 2, Q = 4."""
    @property
    def ring_defended(self) -> ByColour[int]:
        """Ring squares the king's own side attacks, the king itself apart."""
    @property
    def ring_holes(self) -> ByColour[int]:
        """Ring squares the enemy attacks and `ring_defended` does not count."""
    @property
    def escape_squares(self) -> ByColour[int]:
        """Squares adjacent to the king that are empty or capturable and not attacked."""
    @property
    def back_rank_risk(self) -> ByColour[bool]:
        """The king stands on its rank 1 with every forward-adjacent square held by a friendly unit."""
    @property
    def distance(self) -> int:
        """Chebyshev distance between the kings."""
    @property
    def tropism(self) -> ByColour[float]:
        """Mean Chebyshev distance of the enemy's knights, bishops, rooks and queens to this king."""
    @property
    def virtual_mobility(self) -> ByColour[int]:
        """Squares a queen on this king's square would attack."""
    @property
    def ring_defenders(self) -> ByColour[int]:
        """The king's own knights, bishops, rooks and queens attacking its ring."""
    @property
    def ring_defence_weight(self) -> ByColour[int]:
        """Σ over those defenders of N, B = 1, R = 2, Q = 4."""
    @property
    def open_rays(self) -> ByColour[int]:
        """Directions from the king holding at least one square, all of them empty out to the edge."""
    @property
    def luft(self) -> ByColour[bool]:
        """The king stands on its rank 1 with a square ahead of it empty and unattacked."""
    @property
    def castled_side(self) -> ByColour[CastledSide | None]:
        """The side the king's square and its spent castling rights read as."""
    @property
    def opposite_side_castling(self) -> bool:
        """The two kings stand on opposite wings."""
    @property
    def ring_attacker_surplus(self) -> ByColour[int]:
        """The weight besieging each king's ring, less the weight defending it."""
    def to_dict(self) -> KingFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class MobilityFacts:
    """Mobility and space."""

    @property
    def pawns(self) -> ByColour[int]:
        """Squares the side's pawns attack that its own units do not hold."""
    @property
    def knights(self) -> ByColour[int]:
        """The same for its knights."""
    @property
    def bishops(self) -> ByColour[int]:
        """The same for its bishops."""
    @property
    def rooks(self) -> ByColour[int]:
        """The same for its rooks."""
    @property
    def queens(self) -> ByColour[int]:
        """The same for its queens."""
    @property
    def safe_pawns(self) -> ByColour[int]:
        """Of the pawn squares, those no enemy pawn attacks."""
    @property
    def safe_knights(self) -> ByColour[int]:
        """The same for its knights."""
    @property
    def safe_bishops(self) -> ByColour[int]:
        """The same for its bishops."""
    @property
    def safe_rooks(self) -> ByColour[int]:
        """The same for its rooks."""
    @property
    def safe_queens(self) -> ByColour[int]:
        """The same for its queens."""
    @property
    def total(self) -> ByColour[int]:
        """The five counts added up."""
    @property
    def space(self) -> ByColour[int]:
        """Squares the side attacks in the enemy's half."""
    @property
    def centre_control(self) -> ByColour[int]:
        """The side's attacks on d4, e4, d5 and e5."""
    @property
    def extended_centre_control(self) -> ByColour[int]:
        """The side's attacks on c3 to f6."""
    @property
    def immobile_pieces(self) -> ByColour[int]:
        """The side's units other than pawns and kings with no destination."""
    def to_dict(self) -> MobilityFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class AttackFacts:
    """The attack maps and what they say about the units standing on the board."""

    @property
    def controlled(self) -> ByColour[SquareSet]:
        """Every square the side attacks."""
    @property
    def by_pawns(self) -> ByColour[SquareSet]:
        """The squares the side's pawns attack."""
    @property
    def by_knights(self) -> ByColour[SquareSet]:
        """The squares its knights attack."""
    @property
    def by_bishops(self) -> ByColour[SquareSet]:
        """The squares its bishops attack."""
    @property
    def by_rooks(self) -> ByColour[SquareSet]:
        """The squares its rooks attack."""
    @property
    def by_queens(self) -> ByColour[SquareSet]:
        """The squares its queens attack."""
    @property
    def by_kings(self) -> ByColour[SquareSet]:
        """The squares its king attacks."""
    @property
    def attacked(self) -> ByColour[SquareSet]:
        """The side's units the enemy attacks, defended or not; never a king."""
    @property
    def hanging(self) -> ByColour[SquareSet]:
        """Those of them their own side does not defend."""
    @property
    def en_prise(self) -> ByColour[SquareSet]:
        """Those hanging or attacked by a cheaper enemy unit."""
    @property
    def pinned(self) -> ByColour[SquareSet]:
        """The side's units under an absolute pin."""
    @property
    def defended(self) -> ByColour[SquareSet]:
        """The side's units standing on a square their own side attacks."""
    @property
    def attacked_value(self) -> ByColour[int]:
        """The value sum of the attacked units."""
    @property
    def hanging_value(self) -> ByColour[int]:
        """The value sum of the hanging units."""
    @property
    def en_prise_value(self) -> ByColour[int]:
        """The value sum of the units en prise."""
    @property
    def en_prise_max_value(self) -> ByColour[int]:
        """The largest value en prise."""
    @property
    def pinned_value(self) -> ByColour[int]:
        """The value sum of the absolutely pinned units."""
    @property
    def skewer_candidates(self) -> ByColour[int]:
        """Enemy unit pairs the side's sliders skewer."""
    def to_dict(self) -> AttackFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class ExchangeFacts:
    """Each side's captures, judged by the exchange they start. The values for the side not to move are read
    after a null move; when the side to move is in check that move does not exist and they are zero.
    """

    @property
    def see_best_capture(self) -> ByColour[int]:
        """The largest static exchange evaluation over the side's captures; 0 when it has none."""
    @property
    def see_positive_capture_count(self) -> ByColour[int]:
        """Its captures whose evaluation is above 0."""
    @property
    def see_equal_capture_count(self) -> ByColour[int]:
        """Its captures whose evaluation is 0."""
    @property
    def see_positive_total(self) -> ByColour[int]:
        """The sum of the evaluations above 0."""
    def to_dict(self) -> ExchangeFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class ThreatFacts:
    """What each side stands to lose, and the slider geometry behind it. Every set is read on the units it is
    about, and holds no king.
    """

    @property
    def threatened(self) -> ByColour[SquareSet]:
        """The side's units the enemy wins material by taking."""
    @property
    def threatened_value(self) -> ByColour[int]:
        """The value sum of those."""
    @property
    def threat_max_gain(self) -> ByColour[int]:
        """The largest such gain over the side's own units."""
    @property
    def attacked_by_lesser(self) -> ByColour[SquareSet]:
        """The side's units an enemy unit of strictly lower value attacks."""
    @property
    def queen_attacked_by_lesser(self) -> ByColour[bool]:
        """One of those units is a queen."""
    @property
    def overloaded_defenders(self) -> ByColour[SquareSet]:
        """The side's sole defenders of two or more attacked friendly units."""
    @property
    def removable_defenders(self) -> ByColour[SquareSet]:
        """Its sole defenders the enemy can capture without loss."""
    @property
    def loose(self) -> ByColour[SquareSet]:
        """The side's units no unit of their own side defends."""
    @property
    def attacker_surplus(self) -> ByColour[SquareSet]:
        """The side's units more enemy units attack than its own defend."""
    @property
    def xray_through_enemy(self) -> ByColour[int]:
        """The side's x-rays onto an enemy unit through one enemy unit."""
    @property
    def battery_count(self) -> ByColour[int]:
        """The side's batteries: two of its own sliders on one line, agreeing in direction."""
    @property
    def battery_at_king(self) -> ByColour[bool]:
        """One of them bears on a square of the enemy king's ring."""
    def to_dict(self) -> ThreatFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class TacticsFacts:
    """Each side's one-ply tactical options. The values for the side not to move are read after a null move; when
    the side to move is in check that move does not exist and they are zero, which `available` reports.
    """

    @property
    def available(self) -> ByColour[bool]:
        """The side's values could be read."""
    @property
    def check_count(self) -> ByColour[int]:
        """Its moves giving check."""
    @property
    def check_by_pawn(self) -> ByColour[bool]:
        """A pawn move of the side gives check."""
    @property
    def check_by_knight(self) -> ByColour[bool]:
        """A knight move gives check."""
    @property
    def check_by_bishop(self) -> ByColour[bool]:
        """A bishop move gives check."""
    @property
    def check_by_rook(self) -> ByColour[bool]:
        """A rook move gives check."""
    @property
    def check_by_queen(self) -> ByColour[bool]:
        """A queen move gives check."""
    @property
    def safe_check_count(self) -> ByColour[int]:
        """Its checking moves whose destination is safe."""
    @property
    def safe_check_by_pawn(self) -> ByColour[bool]:
        """A pawn move of the side gives check from a safe square."""
    @property
    def safe_check_by_knight(self) -> ByColour[bool]:
        """A knight move does."""
    @property
    def safe_check_by_bishop(self) -> ByColour[bool]:
        """A bishop move does."""
    @property
    def safe_check_by_rook(self) -> ByColour[bool]:
        """A rook move does."""
    @property
    def safe_check_by_queen(self) -> ByColour[bool]:
        """A queen move does."""
    @property
    def double_check_available(self) -> ByColour[bool]:
        """A move gives check from two units at once."""
    @property
    def discovered_check_available(self) -> ByColour[bool]:
        """A move gives check with a unit that did not move."""
    @property
    def mate_in_1(self) -> ByColour[bool]:
        """A move leaves the opponent checkmated."""
    @property
    def stalemate_in_1(self) -> ByColour[bool]:
        """A move leaves the opponent stalemated."""
    @property
    def promotion_files(self) -> ByColour[FileSet]:
        """Files a legal move of the side promotes on."""
    @property
    def promotes_to_queen(self) -> ByColour[bool]:
        """A promotion to a queen is available."""
    @property
    def promotes_to_rook(self) -> ByColour[bool]:
        """One to a rook is."""
    @property
    def promotes_to_bishop(self) -> ByColour[bool]:
        """One to a bishop is."""
    @property
    def promotes_to_knight(self) -> ByColour[bool]:
        """One to a knight is."""
    @property
    def safe_promotion_files(self) -> ByColour[FileSet]:
        """Files a legal promotion with a safe destination lands on."""
    @property
    def capture_count(self) -> ByColour[int]:
        """The side's capturing moves."""
    @property
    def winning_capture_available(self) -> ByColour[bool]:
        """One of them takes a unit that outvalues the capturer or is undefended."""
    @property
    def winning_capture_max_gain(self) -> ByColour[int]:
        """The largest victim value less capturer value over the captures, at least 0."""
    @property
    def captures_hanging(self) -> ByColour[bool]:
        """A capture takes a hanging unit."""
    @property
    def hanging_victim_max_value(self) -> ByColour[int]:
        """The largest value among the hanging units capturable now."""
    @property
    def equal_capture_count(self) -> ByColour[int]:
        """Captures of a defended unit of equal value."""
    @property
    def losing_capture_count(self) -> ByColour[int]:
        """Captures of a defended unit of lower value."""
    @property
    def fork_count(self) -> ByColour[int]:
        """Moves after which the moved unit forks."""
    @property
    def fork_max_value(self) -> ByColour[int]:
        """The largest single forked value."""
    @property
    def knight_fork_available(self) -> ByColour[bool]:
        """A fork by a knight."""
    @property
    def royal_fork_available(self) -> ByColour[bool]:
        """A fork one of whose targets is the king."""
    @property
    def pin_creation_count(self) -> ByColour[int]:
        """Moves creating an absolute or a relative pin."""
    @property
    def skewer_creation_available(self) -> ByColour[bool]:
        """A move creating a skewer."""
    @property
    def discovered_attack_available(self) -> ByColour[bool]:
        """A move uncovering a slider's attack on a unit of value 3 or more."""
    @property
    def legal_move_count(self) -> ByColour[int]:
        """The side's legal moves."""
    @property
    def safe_check_capturing(self) -> ByColour[bool]:
        """A checking move that captures and whose destination is safe."""
    @property
    def discovered_attack_on_queen(self) -> ByColour[bool]:
        """A move uncovering a slider's attack on the enemy queen."""
    @property
    def back_rank_mate_threat(self) -> ByColour[bool]:
        """A move after which a rook or queen of the side checks the enemy king along that king's own rank, its
        back rank being blocked.
        """
    @property
    def quiet_threat_available(self) -> ByColour[bool]:
        """A quiet move after which the largest gain the side threatens is greater than it is now."""
    @property
    def no_safe_moves(self) -> ByColour[bool]:
        """No legal move of the side has a safe destination."""
    @property
    def promotion_see_positive(self) -> ByColour[bool]:
        """A promotion of the side wins material."""
    @property
    def check_available(self) -> ByColour[bool]:
        """Whether the side has a checking move."""

    @property
    def safe_check_available(self) -> ByColour[bool]:
        """Whether it has a checking move with a safe destination."""

    @property
    def promotion_available(self) -> ByColour[bool]:
        """Whether it has a promotion."""

    @property
    def safe_promotion_available(self) -> ByColour[bool]:
        """Whether it has a promotion with a safe destination."""

    @property
    def capture_available(self) -> ByColour[bool]:
        """Whether it has a capture."""

    @property
    def fork_available(self) -> ByColour[bool]:
        """Whether it has a forking move."""

    @property
    def pin_creation_available(self) -> ByColour[bool]:
        """Whether it has a move creating a pin."""

    @property
    def only_moves(self) -> ByColour[bool]:
        """Whether the side has at most two legal moves. False for a side whose values could not be read."""
    def to_dict(self) -> TacticsFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class EndgameFacts:
    """Where the kings stand, how the pawn race runs, and which drawn configuration the material is."""

    @property
    def king_centralisation(self) -> ByColour[int]:
        """Chebyshev distance from the side's king to the nearest central square."""
    @property
    def race_plies(self) -> ByColour[int]:
        """Plies the side's most advanced passer needs to promote unopposed; 8 for a side with no passer."""
    @property
    def opposition(self) -> Opposition | None:
        """The opposition the kings stand in, if any."""
    @property
    def key_square_occupied(self) -> ByColour[bool]:
        """The side's king stands on a key square of one of its own passers."""
    @property
    def wrong_colour_bishop(self) -> ByColour[bool]:
        """The side's bishops all stand on the colour none of its pawns promotes on, every one of those pawns
        being a rook pawn.
        """
    @property
    def drawish_material(self) -> DrawishMaterial | None:
        """The drawn configuration the material is, if it is one of the three."""
    def to_dict(self) -> EndgameFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class MapFacts:
    """The square sets a reader most often wants side by side: what each side attacks, and what of it stands
    hanging, pinned or threatened. Every one of them is also a field of the `attacks` or the `threats` group;
    this group is where they are read together.
    """

    @property
    def controlled(self) -> ByColour[SquareSet]:
        """Every square the side attacks."""
    @property
    def by_pawns(self) -> ByColour[SquareSet]:
        """The squares the side's pawns attack."""
    @property
    def hanging(self) -> ByColour[SquareSet]:
        """The side's units the enemy attacks and their own side does not defend."""
    @property
    def pinned(self) -> ByColour[SquareSet]:
        """The side's units under an absolute pin."""
    @property
    def threatened(self) -> ByColour[SquareSet]:
        """The side's units the enemy wins material by taking."""
    def to_dict(self) -> MapFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class MoveFacts:
    """What one legal move does, beyond what the move itself says. The mover is the side to move; the enemy is
    the other side.
    """

    @property
    def victim(self) -> RoleLetter | None:
        """The role captured, the pawn for an en-passant capture."""
    @property
    def mover(self) -> RoleLetter:
        """The role that moves."""
    @property
    def promotion(self) -> RoleLetter | None:
        """The role a promoting pawn becomes."""
    @property
    def gives_check(self) -> bool:
        """The move gives check."""
    @property
    def gives_safe_check(self) -> bool:
        """The move gives check and its destination is safe."""
    @property
    def is_safe(self) -> bool:
        """The destination is a safe destination."""
    @property
    def captures_hanging(self) -> bool:
        """The move captures a hanging unit."""
    @property
    def escapes_attack(self) -> bool:
        """The origin is attacked by the enemy and the destination is safe."""
    @property
    def to_attacked_by_pawn(self) -> bool:
        """The destination is attacked by an enemy pawn."""
    @property
    def is_castling(self) -> bool:
        """The move is a castling."""
    @property
    def is_en_passant(self) -> bool:
        """The move is an en-passant capture."""
    @property
    def see(self) -> int:
        """The static exchange evaluation of the move, signed."""
    @property
    def threat_created_max(self) -> int:
        """The largest static exchange evaluation of one of the enemy's units after the move; 0 when the move
        threatens nothing.
        """
    @property
    def moves_attacked_unit(self) -> bool:
        """The origin is attacked by the enemy."""
    @property
    def blocks_check(self) -> bool:
        """The destination interposes on the ray of the one unit giving check."""
    @property
    def advances_passer(self) -> bool:
        """The moved unit is a passed pawn."""
    @property
    def creates_passer(self) -> bool:
        """The mover has more passers after the move than before."""
    @property
    def creates_isolated(self) -> bool:
        """The mover has more isolated pawns after the move than before."""
    @property
    def creates_doubled(self) -> bool:
        """The mover has more doubled pawns after the move than before."""
    @property
    def creates_backward(self) -> bool:
        """The mover has more backward pawns after the move than before."""
    @property
    def opens_file_at_enemy_king(self) -> bool:
        """One of the enemy king's shield files carried a pawn of the mover's and carries none after."""
    @property
    def enemy_king_attackers_delta(self) -> int:
        """Change in the number of the mover's units attacking the enemy king ring."""
    @property
    def mover_king_attackers_delta(self) -> int:
        """Change in the number of the enemy's units attacking the mover's king ring."""
    @property
    def mover_hanging_delta(self) -> int:
        """Change in the number of the mover's hanging units."""
    @property
    def enemy_hanging_delta(self) -> int:
        """Change in the number of the enemy's hanging units."""
    @property
    def leaves_unit_hanging(self) -> bool:
        """A square carries a hanging unit of the mover's worth 3 or more after the move and carried none before."""
    @property
    def gives_discovered_attack(self) -> bool:
        """A slider of the mover's that the move leaves standing gains an attack on an enemy unit worth 3 or
        more.
        """
    def to_dict(self) -> MoveFactsDict: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

#: Any one group of facts, as `Facts.group()` hands one over.
type AnyFacts = (
    PlacementFacts
    | StateFacts
    | HistoryFacts
    | MaterialFacts
    | PawnFacts
    | PieceFacts
    | KingFacts
    | MobilityFacts
    | AttackFacts
    | ExchangeFacts
    | ThreatFacts
    | TacticsFacts
    | EndgameFacts
    | MapFacts
)

# --- end of the generated facts section ---

@final
class AnnotatedMove:
    """A legal move and its facts."""

    @property
    def move(self) -> Move: ...
    @property
    def facts(self) -> MoveFacts: ...

@final
class Facts:
    """Every group of facts about one position.

    What each *move* of the position does is a separate answer:
    `Position.annotated_moves()` and `Game.annotated_moves()`. Pickling
    re-reads the facts from the position they were computed for, so the
    history flags a game supplies do not survive a round trip.
    """

    def __new__(cls, fen: str, *, variant: Variant | None = None) -> Facts: ...
    @property
    def variant(self) -> Variant: ...
    @property
    def side_to_move(self) -> Colour: ...
    @property
    def position(self) -> Position: ...
    @staticmethod
    def groups() -> list[str]: ...
    @overload
    def group(self, name: Literal["placement"]) -> PlacementFacts: ...
    @overload
    def group(self, name: Literal["state"]) -> StateFacts: ...
    @overload
    def group(self, name: Literal["history"]) -> HistoryFacts: ...
    @overload
    def group(self, name: Literal["material"]) -> MaterialFacts: ...
    @overload
    def group(self, name: Literal["pawns"]) -> PawnFacts: ...
    @overload
    def group(self, name: Literal["pieces"]) -> PieceFacts: ...
    @overload
    def group(self, name: Literal["king"]) -> KingFacts: ...
    @overload
    def group(self, name: Literal["mobility"]) -> MobilityFacts: ...
    @overload
    def group(self, name: Literal["attacks"]) -> AttackFacts: ...
    @overload
    def group(self, name: Literal["exchange"]) -> ExchangeFacts: ...
    @overload
    def group(self, name: Literal["threats"]) -> ThreatFacts: ...
    @overload
    def group(self, name: Literal["tactics"]) -> TacticsFacts: ...
    @overload
    def group(self, name: Literal["endgame"]) -> EndgameFacts: ...
    @overload
    def group(self, name: Literal["maps"]) -> MapFacts: ...
    @overload
    def group(self, name: str) -> AnyFacts: ...
    def to_dict(self) -> FactsDict: ...
    @property
    def placement(self) -> PlacementFacts: ...
    @property
    def state(self) -> StateFacts: ...
    @property
    def history(self) -> HistoryFacts: ...
    @property
    def material(self) -> MaterialFacts: ...
    @property
    def pawns(self) -> PawnFacts: ...
    @property
    def pieces(self) -> PieceFacts: ...
    @property
    def king(self) -> KingFacts: ...
    @property
    def mobility(self) -> MobilityFacts: ...
    @property
    def attacks(self) -> AttackFacts: ...
    @property
    def exchange(self) -> ExchangeFacts: ...
    @property
    def threats(self) -> ThreatFacts: ...
    @property
    def tactics(self) -> TacticsFacts: ...
    @property
    def endgame(self) -> EndgameFacts: ...
    @property
    def maps(self) -> MapFacts: ...
    def summary(self) -> str: ...

@final
class PgnNode:
    """One move of a game tree."""

    @property
    def move(self) -> Move: ...
    @property
    def san(self) -> str: ...
    @property
    def nags(self) -> list[int]: ...
    @property
    def comment_before(self) -> str: ...
    @property
    def comment_after(self) -> str: ...
    @property
    def variations(self) -> list[list[PgnNode]]: ...

@final
class PgnGame:
    """A game as PGN describes one: tag pairs, a move tree, and a result."""

    def __new__(cls) -> PgnGame: ...
    @staticmethod
    def from_game(game: Game) -> PgnGame: ...
    @property
    def headers(self) -> dict[str, str]: ...
    @property
    def comment(self) -> str: ...
    @comment.setter
    def comment(self, text: str) -> None: ...
    @property
    def result(self) -> str: ...
    @result.setter
    def result(self, marker: str) -> None: ...
    @property
    def variant(self) -> Variant: ...
    @property
    def start_position(self) -> Position: ...
    def set_header(self, name: str, value: str) -> None: ...
    def mainline(self) -> list[PgnNode]: ...
    def game(self) -> Game: ...
    def to_string(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

@final
class PgnReader:
    """An iterator over the games of a PGN source."""

    def __iter__(self) -> PgnReader: ...
    def __next__(self) -> PgnGame: ...

def facts_catalogue() -> list[GroupEntry]: ...
def facts_layout() -> list[dict[str, Any]]: ...
def move_facts_catalogue() -> list[FieldEntry]: ...
def _facts_tensors(
    positions: Iterable[str | Position | Game | Facts],
    *,
    names: Sequence[str],
    packed: bool,
    variant: Variant | None = None,
) -> list[tuple[str, bytearray, str, list[int]]]: ...
def _move_tensors(
    moves: Iterable[AnnotatedMove | MoveFacts],
    *,
    names: Sequence[str],
    packed: bool,
) -> list[tuple[str, bytearray, str, list[int]]]: ...
def pgn_read(
    path: str | os.PathLike[str],
    *,
    skip_errors: bool = False,
) -> PgnReader: ...
def pgn_read_string(text: str, *, skip_errors: bool = False) -> PgnReader: ...
def pgn_count(path: str | os.PathLike[str]) -> int: ...

@final
class PolyglotRaw:
    """One entry as a book file holds it."""

    def __new__(cls, key: int, bits: int, weight: int = 1, learn: int = 0) -> PolyglotRaw: ...
    @property
    def key(self) -> int: ...
    @property
    def bits(self) -> int: ...
    @property
    def uci(self) -> str | None: ...
    @property
    def weight(self) -> int: ...
    @property
    def learn(self) -> int: ...
    def decode(self, position: Position, *, variant: Variant | None = None) -> PolyglotEntry | None: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class PolyglotEntry:
    """One entry whose move has been read against a position."""

    def __new__(cls, key: int, mv: Move, weight: int = 1, learn: int = 0) -> PolyglotEntry: ...
    @property
    def key(self) -> int: ...
    @property
    def move(self) -> Move: ...
    @property
    def bits(self) -> int: ...
    @property
    def weight(self) -> int: ...
    @property
    def learn(self) -> int: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class PolyglotBook:
    """An opening book: entries sorted by key, looked up by position."""

    def __new__(cls, path: str | os.PathLike[str]) -> PolyglotBook: ...
    @staticmethod
    def from_bytes(data: bytes) -> PolyglotBook: ...
    @staticmethod
    def write(path: str | os.PathLike[str], entries: Sequence[PolyglotEntry]) -> None: ...
    def get(self, index: int) -> PolyglotRaw | None: ...
    def raw_entries(self, key: int) -> list[PolyglotRaw]: ...
    def entries(self, position: Position, *, variant: Variant | None = None) -> list[PolyglotEntry]: ...
    def best(self, position: Position, *, variant: Variant | None = None) -> PolyglotEntry | None: ...
    def pick(
        self,
        position: Position,
        seed: int,
        *,
        variant: Variant | None = None,
    ) -> PolyglotEntry | None: ...
    def __len__(self) -> int: ...
    def __iter__(self) -> PolyglotBookIter: ...

@final
class PolyglotBookIter:
    """An iterator over a book's entries, in file order."""

    def __iter__(self) -> PolyglotBookIter: ...
    def __next__(self) -> PolyglotRaw: ...

@final
class PolyglotBuilder:
    """Counts the moves of the games it is given and writes them as a book."""

    def __new__(cls, *, max_ply: int | None = None, min_count: int = 1) -> PolyglotBuilder: ...
    def add_game(self, game: Game) -> None: ...
    def add_pgn(self, path: str | os.PathLike[str]) -> int: ...
    def add_pgn_string(self, text: str) -> int: ...
    def entries(self) -> list[PolyglotRaw]: ...
    def write(self, path: str | os.PathLike[str]) -> None: ...
    def __len__(self) -> int: ...

@final
class Opening:
    """An ECO code and the name that goes with it."""

    @property
    def eco(self) -> str: ...
    @property
    def name(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

def endings_classify(position: Position) -> Ending: ...
def openings_lookup(position: Position) -> Opening | None: ...
def openings_count() -> int: ...

#: What the engine is doing.
type EngineState = Literal["started", "identifying", "idle", "searching", "pondering", "quitting"]

#: What kind of value an engine option takes.
type OptionType = Literal["check", "spin", "combo", "button", "string"]

#: Whether a score is exact, or a bound the search did not resolve.
type ScoreBound = Literal["lowerbound", "upperbound"]

#: What one line an engine sent says.
type MessageKind = Literal[
    "id",
    "uciok",
    "readyok",
    "option",
    "info",
    "bestmove",
    "registration",
    "copyprotection",
    "raw",
]

class UciError(Exception):
    """The base of every error talking to an engine raises."""

class EngineTimeout(UciError):
    """The engine did not say what was awaited in time."""

class EngineDied(UciError):
    """The engine closed its output or exited."""

class ProtocolError(UciError):
    """The engine broke the order of the conversation."""

@final
class Limits:
    """What bounds a search; times are seconds."""

    def __new__(
        cls,
        *,
        depth: int | None = None,
        nodes: int | None = None,
        movetime: float | None = None,
        mate: int | None = None,
        infinite: bool = False,
        ponder: bool = False,
        white_time: float | None = None,
        black_time: float | None = None,
        white_increment: float | None = None,
        black_increment: float | None = None,
        moves_to_go: int | None = None,
        search_moves: Sequence[str] | None = None,
    ) -> Limits: ...
    @property
    def depth(self) -> int | None: ...
    @property
    def nodes(self) -> int | None: ...
    @property
    def movetime(self) -> float | None: ...
    @property
    def mate(self) -> int | None: ...
    @property
    def infinite(self) -> bool: ...
    @property
    def ponder(self) -> bool: ...
    @property
    def white_time(self) -> float | None: ...
    @property
    def black_time(self) -> float | None: ...
    @property
    def white_increment(self) -> float | None: ...
    @property
    def black_increment(self) -> float | None: ...
    @property
    def moves_to_go(self) -> int | None: ...
    @property
    def search_moves(self) -> list[str]: ...

@final
class Option:
    """One option an engine offers, and the domain it declared for it."""

    @property
    def name(self) -> str: ...
    @property
    def type(self) -> OptionType: ...
    @property
    def default(self) -> bool | int | str | None: ...
    @property
    def min(self) -> int | None: ...
    @property
    def max(self) -> int | None: ...
    @property
    def vars(self) -> list[str]: ...
    def value_text(self, value: bool | int | str | None = None) -> str | None: ...

@final
class Info:
    """One `info` line of a search, its moves read against the position searched."""

    @property
    def depth(self) -> int | None: ...
    @property
    def seldepth(self) -> int | None: ...
    @property
    def time(self) -> float | None: ...
    @property
    def nodes(self) -> int | None: ...
    @property
    def nps(self) -> int | None: ...
    @property
    def pv(self) -> list[Move]: ...
    @property
    def multipv(self) -> int | None: ...
    @property
    def cp(self) -> int | None: ...
    @property
    def mate(self) -> int | None: ...
    @property
    def bound(self) -> ScoreBound | None: ...
    @property
    def wdl(self) -> tuple[int, int, int] | None: ...
    @property
    def currmove(self) -> Move | None: ...
    @property
    def currmovenumber(self) -> int | None: ...
    @property
    def hashfull(self) -> int | None: ...
    @property
    def tbhits(self) -> int | None: ...
    @property
    def sbhits(self) -> int | None: ...
    @property
    def cpuload(self) -> int | None: ...
    @property
    def refutation(self) -> list[Move]: ...
    @property
    def currline(self) -> list[Move]: ...
    @property
    def currline_cpu(self) -> int | None: ...
    @property
    def string(self) -> str | None: ...
    @property
    def unknown(self) -> list[str]: ...

@final
class Answer:
    """An engine's answer to a search."""

    @property
    def best(self) -> Move | None: ...
    @property
    def ponder(self) -> Move | None: ...

@final
class Command:
    """One line a client sends to an engine."""

    @staticmethod
    def uci() -> Command: ...
    @staticmethod
    def debug(on: bool) -> Command: ...
    @staticmethod
    def isready() -> Command: ...
    @staticmethod
    def setoption(name: str, value: str | None = None) -> Command: ...
    @staticmethod
    def ucinewgame() -> Command: ...
    @staticmethod
    def position(game: Game, castling: CastlingStyle | None = None) -> Command: ...
    @staticmethod
    def go(limits: Limits | None = None) -> Command: ...
    @staticmethod
    def stop() -> Command: ...
    @staticmethod
    def ponderhit() -> Command: ...
    @staticmethod
    def quit() -> Command: ...
    def to_line(self) -> str: ...
    @property
    def keyword(self) -> str: ...

@final
class Message:
    """One line an engine sent, read into what it says."""

    @property
    def kind(self) -> MessageKind: ...
    @property
    def line(self) -> str: ...
    @property
    def key(self) -> str | None: ...
    @property
    def value(self) -> str | None: ...
    @property
    def status(self) -> str | None: ...
    @property
    def option(self) -> Option | None: ...
    @property
    def info(self) -> Info | None: ...
    @property
    def answer(self) -> Answer | None: ...

@final
class Session:
    """Which commands may go out and which messages may come in."""

    def __new__(cls) -> Session: ...
    @property
    def state(self) -> EngineState: ...
    @property
    def pending_ready(self) -> int: ...
    def sent(self, command: Command) -> None: ...
    def received(self, message: Message) -> None: ...

def uci_parse(line: str, game: Game | None = None) -> Message: ...

@final
class Search:
    """A search in flight: an iterator over its reports."""

    @property
    def done(self) -> bool: ...
    def answer(self) -> Answer: ...
    def stop(self) -> None: ...
    def ponderhit(self) -> None: ...
    def __iter__(self) -> Search: ...
    def __next__(self) -> Info: ...
    def __enter__(self) -> Search: ...
    def __exit__(self, *args: object) -> None: ...

@final
class Engine:
    """A UCI engine process, addressed with blocking calls."""

    def __new__(
        cls,
        command: str | os.PathLike[str],
        args: Sequence[str] = (),
        *,
        cwd: str | os.PathLike[str] | None = None,
        timeout: float = 10.0,
    ) -> Engine: ...
    @property
    def timeout(self) -> float: ...
    @timeout.setter
    def timeout(self, timeout: float) -> None: ...
    @property
    def name(self) -> str | None: ...
    @property
    def author(self) -> str | None: ...
    @property
    def options(self) -> dict[str, Option]: ...
    @property
    def state(self) -> EngineState: ...
    @property
    def is_alive(self) -> bool: ...
    def handshake(self) -> None: ...
    def option(self, name: str) -> Option | None: ...
    def set_option(self, name: str, value: bool | int | str | None = None) -> None: ...
    def debug(self, on: bool) -> None: ...
    def new_game(self) -> None: ...
    def is_ready(self) -> None: ...
    def set_position(self, game: Game) -> None: ...
    def go(self, limits: Limits | None = None, *, timeout: float | None = None) -> Search: ...
    def play(self, game: Game, limits: Limits | None = None, *, timeout: float | None = None) -> Answer: ...
    def analyse(
        self,
        game: Game,
        limits: Limits | None = None,
        *,
        multipv: int | None = None,
        timeout: float | None = None,
    ) -> list[Info]: ...
    def send_line(self, text: str) -> None: ...
    def next_line(self, timeout: float | None = None) -> str | None: ...
    def quit(self) -> int | None: ...
    def kill(self) -> None: ...
    def __enter__(self) -> Engine: ...
    def __exit__(self, *args: object) -> None: ...
