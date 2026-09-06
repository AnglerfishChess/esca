"""Each tool called in process, on positions derived by hand."""

import esca
import pytest

from chess_esca_mcp import server
from tests import positions

#: Every fact group, in the order `docs/features.md` defines them.
GROUPS = [
    "placement",
    "state",
    "history",
    "material",
    "pawns",
    "pieces",
    "king",
    "mobility",
    "attacks",
    "exchange",
    "threats",
    "tactics",
    "endgame",
    "maps",
]


def reasons(answer: dict) -> list[str]:
    """The reason names an illegal move came back with, in the order it lists them."""
    return [item["reason"] for item in answer["reasons"]]


def written_book(path, weights: dict[str, int]):
    """A Polyglot book holding the start position, `weights` naming its moves in SAN."""
    game = esca.Game()
    moves = {game.move_to_san(mv): mv for mv in game.legal_moves()}
    esca.polyglot.Book.write(
        path,
        [esca.polyglot.Entry(game.position.polyglot_key, moves[san], weight) for san, weight in weights.items()],
    )
    return path


def obstacles(answer: dict, side: str, wing: str) -> list[str]:
    """The reason names standing in the way of one castling.

    In the order the fields behind them are declared: the missing right or
    rook, the king in check, the attacked path, the blocked path.
    """
    return [item["reason"] for item in answer["castling"][side][wing]["obstacles"]]


class TestPosition:
    def test_start_position_is_read_whole(self):
        answer = server.position()
        assert answer["fen"] == positions.START
        assert answer["side_to_move"] == "white"
        assert answer["legal_move_count"] == 20
        assert answer["status"]["state"] == "in_progress"
        assert answer["check"]["in_check"] is False
        assert answer["material"]["white"]["pawns"] == 8
        assert answer["material"]["balance"] == 0

    def test_moves_are_played_from_the_position(self):
        answer = server.position(moves=["e4", "e5", "Nf3"])
        assert answer["ply"] == 3
        assert answer["line"] == ["e4", "e5", "Nf3"]
        assert answer["side_to_move"] == "black"
        assert answer["opening"] == {"eco": "C40", "name": "King's Knight Opening"}

    @pytest.mark.parametrize(
        ("wing", "expected"),
        [("short", ["path_attacked"]), ("long", [])],
        ids=["short, whose f1 the bishop covers", "long, which the bishop does not reach"],
    )
    def test_an_attacked_path_stops_one_castling_only(self, wing, expected):
        answer = server.position(fen=positions.CASTLING_ATTACKED)
        assert obstacles(answer, "white", wing) == expected

    def test_a_blocked_castling_names_the_unit_in_the_way(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)["castling"]["white"]["short"]
        [blocked] = answer["obstacles"]
        assert blocked["reason"] == "path_blocked"
        assert blocked["occupants"] == [{"square": "f1", "colour": "white", "role": "bishop"}]
        assert answer["king_lands_on"] == "g1"

    def test_a_blocked_castling_names_every_unit_in_the_way(self):
        answer = server.position(fen=positions.CASTLING_PATH_BLOCKED_TWICE)["castling"]["white"]
        [blocked] = answer["long"]["obstacles"]
        assert blocked["occupants"] == [
            {"square": "b1", "colour": "white", "role": "knight"},
            {"square": "d1", "colour": "white", "role": "queen"},
        ]
        assert answer["short"]["obstacles"] == []

    def test_a_castling_stopped_every_way_answers_with_every_obstacle(self):
        answer = server.position(fen=positions.CASTLING_STOPPED_EVERY_WAY)
        assert obstacles(answer, "white", "short") == ["king_in_check", "path_attacked", "path_blocked"]
        assert answer["castling"]["white"]["short"]["allowed"] is False

    def test_an_en_passant_offer_lists_the_pawn_beside_it(self):
        answer = server.position(fen=positions.EN_PASSANT)["en_passant"]
        assert answer["target"] == "d6"
        assert answer["available"] is True
        assert answer["captures"] == [{"origin": "e5", "legal": True, "forbidden_by": None}]

    def test_an_en_passant_offer_lists_every_pawn_beside_it(self):
        answer = server.position(fen=positions.EN_PASSANT_TWO_TAKERS)["en_passant"]
        assert answer["available"] is True
        assert answer["captures"] == [
            {"origin": "c5", "legal": True, "forbidden_by": None},
            {"origin": "e5", "legal": True, "forbidden_by": None},
        ]

    def test_an_en_passant_capture_that_exposes_the_king_says_so(self):
        answer = server.position(fen=positions.EN_PASSANT_EXPOSES_KING)["en_passant"]
        assert answer["available"] is False
        [capture] = answer["captures"]
        assert capture["forbidden_by"]["reason"] == "exposes_king"
        assert capture["forbidden_by"]["attacker"] == "h5"

    def test_fools_mate_is_a_finished_game(self):
        answer = server.position(moves=positions.FOOLS_MATE)
        assert answer["status"]["state"] == "checkmate"
        assert answer["status"]["winner"] == "black"
        assert answer["status"]["result"] == "0-1"
        assert answer["legal_move_count"] == 0
        assert answer["check"]["checkers"] == [{"square": "h4", "colour": "black", "role": "queen"}]

    def test_a_double_check_names_both_units_giving_it(self):
        answer = server.position(fen=positions.DOUBLE_CHECK)["check"]
        assert answer["double_check"] is True
        assert answer["king"] == "e1"
        # Board order, rank 1 first: f3 before e8, as esca's own sentence says it.
        assert answer["checkers"] == [
            {"square": "f3", "colour": "black", "role": "knight"},
            {"square": "e8", "colour": "black", "role": "rook"},
        ]

    def test_a_threefold_repetition_is_claimable_with_its_plies(self):
        answer = server.position(moves=positions.THREEFOLD)["status"]
        assert answer["claims"] == ["threefold_repetition"]
        [claimable] = answer["claimable"]
        assert claimable["kind"] == "threefold"
        assert claimable["repetition"]["count"] == 3
        assert claimable["repetition"]["plies"] == [0, 4, 8]
        assert claimable["repetition"]["near_misses"] == []

    def test_every_claim_that_stands_is_answered_with_its_own_evidence(self):
        answer = server.position(fen=positions.FIFTY_MOVE_EDGE, moves=positions.SHUFFLE)["status"]
        assert answer["claims"] == ["fifty_moves", "threefold_repetition"]
        assert [claim["kind"] for claim in answer["claimable"]] == ["threefold", "fifty_moves"]
        threefold, fifty = answer["claimable"]
        assert threefold["repetition"]["count"] == 3
        assert threefold["repetition"]["plies"] == [0, 4, 8]
        assert fifty["fifty_move"]["clock"] == 106
        assert fifty["fifty_move"]["plies_to_claim"] == 0
        assert fifty["fifty_move"]["plies_to_automatic"] == 44

    def test_repetition_is_invisible_without_the_moves(self):
        played = server.position(moves=positions.THREEFOLD)
        alone = server.position(fen=played["fen"])
        assert alone["status"]["claims"] == []
        assert alone["status"]["history_known"] is False

    @pytest.mark.parametrize(
        ("fen", "kind"),
        [
            (positions.STALEMATE, "stalemate"),
            (positions.INSUFFICIENT_MATERIAL, "insufficient_material"),
        ],
        ids=["a king with no move", "two kings alone"],
    )
    def test_an_automatic_draw_is_named_and_evidenced(self, fen, kind):
        answer = server.position(fen=fen)["status"]
        assert answer["state"] == kind
        assert answer["result"] == "1/2-1/2"
        assert [draw["kind"] for draw in answer["automatic"]] == [kind]

    def test_every_automatic_draw_that_holds_is_named(self):
        answer = server.position(fen=positions.STALEMATE_AND_INSUFFICIENT)["status"]
        assert answer["state"] == "stalemate"
        assert [drawn["kind"] for drawn in answer["automatic"]] == ["stalemate", "insufficient_material"]
        stuck, material = answer["automatic"]
        assert stuck["stalemate"]["king"] == "h8"
        assert material["material"] == "kb_v_k"

    def test_the_stalemate_evidence_names_every_square_the_king_is_denied(self):
        [drawn] = server.position(fen=positions.STALEMATE)["status"]["automatic"]
        assert drawn["stalemate"]["king"] == "h8"
        # esca lists the escape squares in ascending square order: rank 7 before rank 8.
        assert drawn["stalemate"]["escape_squares"] == [
            {"square": "g7", "attackers": ["g6", "f7"]},
            {"square": "h7", "attackers": ["g6", "f7"]},
            {"square": "g8", "attackers": ["f7"]},
        ]
        assert drawn["stalemate"]["stuck_units"] == []

    def test_two_pins_at_once_are_both_answered_with_their_rays(self):
        answer = server.position(fen=positions.TWO_PINS)["pins"]
        assert [(item["pinned"], item["pinner"], item["king"]) for item in answer["white"]] == [
            ("c3", "a5", "e1"),
            ("e4", "e8", "e1"),
        ]
        knight, bishop = answer["white"]
        assert knight["ray"] == ["d2", "c3", "b4"]
        assert bishop["ray"] == ["e2", "e3", "e4", "e5", "e6", "e7"]
        assert answer["black"] == []

    def test_two_skewers_at_once_are_both_answered_with_their_rays(self):
        answer = server.position(fen=positions.TWO_SKEWERS)["skewers"]
        assert [(item["attacker"], item["front"], item["behind"]) for item in answer["black"]] == [
            ("f3", "d5", "c6"),
            ("h4", "h6", "h7"),
        ]
        on_the_diagonal, on_the_file = answer["black"]
        assert on_the_diagonal["ray"] == ["e4", "d5"]
        assert on_the_file["ray"] == ["h5", "h6"]
        assert answer["white"] == []

    @pytest.mark.parametrize(
        ("fen", "kind"),
        [(positions.NOT_A_FEN, "invalid_fen"), (positions.NO_KINGS, "invalid_fen")],
        ids=["text that is no FEN", "a FEN of no legal position"],
    )
    def test_a_fen_that_names_no_position_is_an_error_object(self, fen, kind):
        answer = server.position(fen=fen)
        assert answer["error"]["kind"] == kind
        assert answer["error"]["fen"] == fen
        assert "hint" in answer["error"]

    def test_an_unknown_variant_lists_the_known_ones(self):
        answer = server.position(variant="atomic")
        assert answer["error"]["kind"] == "unknown_variant"
        assert "chess960" in answer["error"]["hint"]

    def test_chess960_starts_where_its_own_rules_say(self):
        answer = server.position(variant="chess960")
        assert answer["variant"] == "chess960"
        assert answer["legal_move_count"] > 0


class TestLegalMoves:
    def test_the_start_position_has_twenty(self):
        answer = server.legal_moves()
        assert answer["count"] == 20
        assert [move["san"] for move in answer["moves"]] == positions.START_MOVES

    def test_every_move_carries_the_role_that_makes_it_and_what_it_meets(self):
        answer = server.legal_moves(fen=positions.DOUBLE_CHECK)
        assert answer["moves"] == [
            {
                "san": san,
                "uci": uci,
                "kind": "quiet",
                "mover": "king",
                "capture": False,
                "victim": None,
                "promotion": None,
                "gives_check": False,
                "see": 0,
            }
            for san, uci in (("Kd1", "e1d1"), ("Kf1", "e1f1"), ("Kf2", "e1f2"))
        ]

    def test_nothing_but_a_quiet_move_stands_at_the_start(self):
        assert server.legal_moves()["by_category"] == {
            "captures": [],
            "checks": [],
            "castling": [],
            "promotions": [],
            "en_passant": [],
            "quiet": positions.START_MOVES,
        }

    def test_categories_hold_what_they_are_named_for(self):
        answer = server.legal_moves(fen=positions.CASTLING_ATTACKED)["by_category"]
        assert answer["castling"] == ["O-O-O"]
        assert answer["checks"] == ["Rxa8+", "Rxh8+"]
        assert answer["captures"] == ["Rxa8+", "Rxh8+"]
        assert answer["promotions"] == []
        assert answer["en_passant"] == []

    def test_a_move_that_is_two_things_at_once_stands_under_both(self):
        answer = server.legal_moves(fen=positions.EN_PASSANT_TWO_TAKERS)["by_category"]
        assert answer["captures"] == ["cxd6", "exd6"]
        assert answer["en_passant"] == ["cxd6", "exd6"]
        assert answer["quiet"] == ["Kd1", "Kd2", "Ke2", "Kf1", "Kf2", "c6", "e6"]

    def test_categories_can_be_left_out(self):
        assert "by_category" not in server.legal_moves(categories=False)

    def test_an_en_passant_capture_is_listed_as_one(self):
        answer = server.legal_moves(fen=positions.EN_PASSANT)
        assert answer["by_category"]["en_passant"] == ["exd6"]
        [capture] = [move for move in answer["moves"] if move["kind"] == "en_passant"]
        assert capture["san"] == "exd6"
        assert capture["victim"] == "pawn"

    def test_a_finished_game_has_none(self):
        assert server.legal_moves(moves=positions.FOOLS_MATE)["count"] == 0


class TestExplainMove:
    def test_a_legal_move_says_what_it_changes(self):
        answer = server.explain_move(move="e4")
        assert answer["legal"] is True
        assert answer["uci"] == "e2e4"
        assert answer["after"]["fen"].startswith("rnbqkbnr/pppppppp/8/8/4P3")
        assert answer["effects"]["gives_check"] is False

    def test_a_move_may_be_written_as_uci_or_as_san(self):
        assert server.explain_move(move="e2e4")["san"] == server.explain_move(move="e4")["san"]

    def test_a_mating_move_leaves_a_finished_game_behind(self):
        answer = server.explain_move(move="Qh4#", moves=positions.FOOLS_MATE[:3])
        assert answer["legal"] is True
        assert answer["after"]["status"]["state"] == "checkmate"
        assert answer["after"]["legal_move_count"] == 0

    def test_a_move_that_would_repeat_names_the_claim_it_opens(self):
        answer = server.explain_move(move="Ng8", moves=positions.THREEFOLD[:7])
        assert [claim["kind"] for claim in answer["claims_after"]] == ["threefold"]

    def test_a_move_that_opens_two_claims_names_them_both(self):
        answer = server.explain_move(move="Ka8", fen=positions.FIFTY_MOVE_EDGE, moves=positions.SHUFFLE[:-1])
        assert [claim["kind"] for claim in answer["claims_after"]] == ["threefold", "fifty_moves"]
        threefold, fifty = answer["claims_after"]
        assert threefold["repetition"]["plies"] == [0, 4, 8]
        assert fifty["fifty_move"]["plies_to_claim"] == 0

    def test_an_impossible_move_names_the_unit_and_what_it_could_play(self):
        answer = server.explain_move(move="e2e5")
        assert answer["legal"] is False
        assert reasons(answer) == ["not_a_move_of_this_unit"]
        assert answer["legal_moves_from_origin"] == ["e3", "e4"]

    def test_an_empty_origin_is_named_as_one(self):
        answer = server.explain_move(move="e4e5")
        assert reasons(answer) == ["empty_origin"]

    def test_a_move_of_the_other_side_is_named_as_one(self):
        answer = server.explain_move(move="e7e5")
        assert reasons(answer) == ["not_the_side_to_move"]

    def test_a_castling_the_enemy_covers_names_the_square_and_who_covers_it(self):
        answer = server.explain_move(move="O-O", fen=positions.CASTLING_ATTACKED)
        assert reasons(answer) == ["path_attacked"]
        [covered] = answer["reasons"]
        [attacked] = covered["squares"]
        assert attacked == {"square": "f1", "attackers": [{"square": "b5", "colour": "black", "role": "bishop"}]}

    def test_a_forbidden_castling_answers_with_every_obstacle_at_once(self):
        answer = server.explain_move(move="O-O", fen=positions.CASTLING_STOPPED_EVERY_WAY)
        assert reasons(answer) == ["king_in_check", "path_attacked", "path_blocked"]
        checked, attacked, blocked = answer["reasons"]
        assert checked["king"] == "e1"
        assert checked["attackers"] == [{"square": "e8", "colour": "black", "role": "rook"}]
        assert attacked["squares"] == [
            {"square": "f1", "attackers": [{"square": "b5", "colour": "black", "role": "bishop"}]},
            {"square": "g1", "attackers": [{"square": "g8", "colour": "black", "role": "rook"}]},
        ]
        assert blocked["occupants"] == [{"square": "g1", "colour": "white", "role": "knight"}]
        assert answer["legal_moves"] == ["Kd1", "Kd2", "Kf2", "Ne2"]

    def test_a_castling_without_the_right_says_so(self):
        answer = server.explain_move(move="O-O", fen="r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1")
        assert reasons(answer) == ["no_castling_right"]

    @pytest.mark.parametrize(
        ("move", "named"),
        [
            ("e5d6", ["exposes_king"]),
            ("exd6", ["exposes_king", "no_legal_move_to_destination"]),
        ],
        ids=["as UCI, read from the pawn on e5", "as SAN, read from the destination"],
    )
    def test_an_en_passant_capture_that_exposes_the_king_says_which_unit_waits(self, move, named):
        answer = server.explain_move(move=move, fen=positions.EN_PASSANT_EXPOSES_KING)
        assert answer["legal"] is False
        assert reasons(answer) == named
        exposed = answer["reasons"][named.index("exposes_king")]
        assert exposed["capture"] == "en_passant"
        assert exposed["origin"] == "e5"
        assert exposed["attacker"] == "h5"

    def test_a_promotion_left_unnamed_is_named_as_the_reason(self):
        answer = server.explain_move(move="a7a8", fen="8/P6k/8/8/8/8/8/K7 w - - 0 1")
        assert reasons(answer) == ["promotion_not_named"]
        [unnamed] = answer["reasons"]
        assert unnamed["promotion_roles"] == ["queen", "rook", "bishop", "knight"]
        assert answer["legal_moves_from_origin"] == ["a8=B", "a8=N", "a8=Q", "a8=R"]

    def test_text_that_is_no_move_at_all_says_so(self):
        answer = server.explain_move(move="zzz")
        assert reasons(answer) == ["unreadable_move_text"]

    def test_a_bad_position_is_an_error_before_the_move_is_read(self):
        assert server.explain_move(move="e4", fen=positions.NOT_A_FEN)["error"]["kind"] == "invalid_fen"


class TestFacts:
    def test_the_default_leaves_out_the_two_groups_that_restate_others(self):
        answer = server.facts()
        assert answer["groups_available"] == GROUPS
        assert list(answer["groups"]) == [name for name in GROUPS if name not in ("placement", "maps")]

    def test_only_the_named_groups_come_back_in_the_catalogue_order(self):
        answer = server.facts(groups=["material", "state"])
        assert list(answer["groups"]) == ["state", "material"]

    def test_every_side_paired_value_is_labelled_by_colour(self):
        answer = server.facts(groups=["material"])["groups"]["material"]
        assert answer["pawns"] == {"white": 8, "black": 8}
        assert answer["value"] == {"white": 39, "black": 39}

    def test_the_answer_says_who_is_to_move_and_nothing_is_read_against_it(self):
        assert server.facts()["side_to_move"] == "white"
        black = server.facts(moves=["e4"])
        assert black["side_to_move"] == "black"
        assert black["groups"]["material"]["pawns"] == {"white": 8, "black": 8}

    def test_a_group_names_the_units_of_each_colour(self):
        answer = server.facts(groups=["placement"])["groups"]["placement"]
        assert answer["kings"]["white"] == ["e1"]
        assert answer["pawns"]["black"] == ["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"]

    def test_a_run_of_values_is_a_list_in_the_order_its_note_states(self):
        answer = server.facts(groups=["pawns"])
        assert answer["groups"]["pawns"]["count_by_file"]["white"] == [1] * 8
        assert answer["field_notes"]["pawns.count_by_file"] == "The side's pawns per file, file a first."

    def test_the_king_shield_points_at_the_files_it_stands_on(self):
        answer = server.facts(groups=["king"])
        assert answer["groups"]["king"]["shield_files"]["white"] == ["d", "e", "f"]
        assert answer["groups"]["king"]["shield"]["white"] == [1, 1, 1]
        assert "shield_files` names them" in answer["field_notes"]["king.shield"]

    def test_only_the_runs_of_the_named_groups_carry_a_note(self):
        answer = server.facts(groups=["material"])
        assert answer["field_notes"] == {}

    def test_an_unknown_group_lists_the_known_ones(self):
        answer = server.facts(groups=["nope"])
        assert answer["error"]["kind"] == "unknown_fact_group"
        assert answer["error"]["groups_available"] == GROUPS

    def test_a_bad_position_is_an_error_object(self):
        assert server.facts(fen=positions.NOT_A_FEN)["error"]["kind"] == "invalid_fen"


class TestEnding:
    def test_a_rook_against_a_lone_king_is_won_by_the_box_method(self):
        answer = server.ending(fen=positions.KING_AND_ROOK_V_KING)
        assert answer["is_ending"] is True
        assert answer["class"] == "kr_v_k"
        assert answer["verdict"] == {"kind": "win", "colour": "white"}
        assert answer["technique"] == "box_method"

    def test_the_signature_writes_the_stronger_side_first_and_counts_both(self):
        answer = server.ending(fen=positions.KING_AND_ROOK_V_KING)["signature"]
        assert answer["text"] == "KRvK"
        assert answer["stronger"] == "white"
        assert answer["count"]["white"] == {
            "pawn": 0,
            "knight": 0,
            "bishop": 0,
            "rook": 1,
            "queen": 0,
            "king": 1,
        }
        assert answer["count"]["black"] == {
            "pawn": 0,
            "knight": 0,
            "bishop": 0,
            "rook": 0,
            "queen": 0,
            "king": 1,
        }
        assert answer["pieces"] == {"white": 1, "black": 0}

    def test_a_bishop_of_the_wrong_colour_draws_the_pawn_away(self):
        answer = server.ending(fen=positions.WRONG_BISHOP)
        assert answer["class"] == "kbp_v_k"
        assert answer["verdict"] == {"kind": "draw", "colour": None}
        assert answer["technique"] == "wrong_bishop"

    def test_the_evidence_stays_grouped_the_way_esca_groups_it(self):
        answer = server.ending(fen=positions.WRONG_BISHOP)["evidence"]
        assert answer["bishops"] == {"opposite_colours": False, "same_colour": True, "wrong_bishop": True}
        assert answer["pawn"]["pawn"] == "a2"
        assert answer["pawn"]["promotion"] == "a8"
        assert answer["pawn"]["rook_pawn"] is True
        assert answer["pawn"]["defender_inside_square"] is True
        assert answer["opposition"] is False
        # The wording is esca's own, one sentence per group of evidence it holds.
        assert answer["prose"] == [
            "White's pawn on a2 is 5 moves from promoting on a8, and the defending king is "
            "inside the square of the pawn. The bishop stands on the square colour its rook "
            "pawn does not promote on, so it can never cover the promotion square."
        ]

    def test_a_middlegame_is_no_ending_and_still_says_what_stands_there(self):
        answer = server.ending(fen=positions.MIDDLEGAME)
        assert answer["is_ending"] is False
        assert answer["class"] == "not_an_ending"
        assert answer["technique"] == "none"
        assert answer["signature"]["count"]["black"]["pawn"] == 8
        assert answer["evidence"]["pawn"] is None

    def test_the_whole_position_names_the_ending_without_its_evidence(self):
        answer = server.position(fen=positions.WRONG_BISHOP)["ending"]
        assert answer["class"] == "kbp_v_k"
        assert answer["technique"] == "wrong_bishop"
        assert answer["prose"] == server.ending(fen=positions.WRONG_BISHOP)["prose"]
        assert set(answer) == {"class", "verdict", "technique", "prose"}

    def test_a_position_that_is_no_ending_carries_none(self):
        assert "ending" not in server.position(fen=positions.MIDDLEGAME)

    def test_a_bad_position_is_an_error_object(self):
        assert server.ending(fen=positions.NOT_A_FEN)["error"]["kind"] == "invalid_fen"


class TestProse:
    """The sentences are esca's own `describe()` wording, quoted as it stands."""

    def test_a_blocked_castling_says_in_words_what_blocks_it(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)["castling"]["white"]
        assert answer["short"]["prose"] == ["White cannot castle short: the path is blocked on f1."]
        assert answer["long"]["prose"] == ["White cannot castle long: the path is blocked on b1."]

    def test_a_castling_stopped_every_way_says_every_obstacle_in_one_sentence(self):
        answer = server.position(fen=positions.CASTLING_STOPPED_EVERY_WAY)["castling"]["white"]
        assert answer["short"]["prose"] == [
            "White cannot castle short: the king is in check from e8, and a king may not castle "
            "out of check; the enemy covers f1 and g1, which the king must cross or land on; "
            "the path is blocked on g1."
        ]

    def test_the_gathered_sentences_are_every_sentence_below_them_once(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)
        assert answer["prose"] == [
            "No draw condition holds in this position.",
            "White cannot castle short: the path is blocked on f1.",
            "White cannot castle long: the path is blocked on b1.",
            "Black may castle short: nothing stands in its way.",
            "Black may castle long: nothing stands in its way.",
            "No pawn has just advanced two squares, so no en-passant capture is on offer.",
        ]
        assert len(answer["prose"]) == len(set(answer["prose"]))

    def test_a_pin_says_in_words_what_holds_the_unit(self):
        answer = server.position(fen=positions.PIN_ON_THE_BACK_RANK)
        [pinned] = answer["pins"]["white"]
        [said] = pinned["prose"]
        assert said == "The unit on e1 may not move off the line between the enemy unit on g1 and its own king on d1."
        assert said in answer["prose"]

    def test_the_draw_status_speaks_once_for_all_the_conditions_under_it(self):
        answer = server.position(fen=positions.STALEMATE)["status"]
        assert answer["prose"] == [
            "The game is drawn already. The side to move is not in check and has no legal move, "
            "which is stalemate: a draw."
        ]
        [drawn] = answer["automatic"]
        assert "prose" not in drawn
        assert drawn["stalemate"]["prose"] == [
            "The king on h8 is the only unit of the side to move, and every square it could step to is covered."
        ]

    def test_an_illegal_castling_is_explained_in_words(self):
        answer = server.explain_move(move="O-O", fen=positions.CASTLING_ATTACKED)
        assert answer["legal"] is False
        assert answer["prose"] == [
            "White cannot castle short: the enemy covers f1, which the king must cross or land on."
        ]

    def test_an_en_passant_obstacle_names_the_waiting_unit_in_words(self):
        answer = server.explain_move(move="e5d6", fen=positions.EN_PASSANT_EXPOSES_KING)
        [obstacle] = answer["reasons"]
        assert obstacle["prose"] == ["Both pawns leave the rank at once, which uncovers the king to the unit on h5."]
        assert answer["prose"] == obstacle["prose"]

    def test_a_legal_move_carries_the_sentences_of_what_it_leaves_behind(self):
        answer = server.explain_move(move="Ng8", moves=positions.THREEFOLD[:7])
        [claim] = answer["claims_after"]
        [said] = claim["prose"]
        assert said == "The position has stood three times, so either player may claim a draw."
        assert said in answer["prose"]


class TestOpening:
    def test_a_named_position_is_named(self):
        answer = server.opening(moves=["e4", "c5"])
        assert answer["at_position"] == {"eco": "B20", "name": "Sicilian Defense"}
        assert answer["line"] == ["e4", "c5"]

    def test_the_start_position_has_no_name(self):
        assert server.opening()["at_position"] is None

    def test_a_line_keeps_the_last_name_it_reached(self):
        answer = server.opening(moves=["e4", "c5", "a3", "a6", "h3", "h6"])
        assert answer["at_position"] is None
        assert answer["reached"]["name"].startswith("Sicilian")


class TestBookMoves:
    def test_a_book_that_is_not_there_says_where_to_get_one(self):
        answer = server.book_moves(book="/no/such/book.bin")
        assert answer["error"]["kind"] == "book_unreadable"
        assert "polyglot" in answer["error"]["hint"].lower()

    def test_a_book_written_here_answers_the_position_it_holds(self, tmp_path):
        book = written_book(tmp_path / "three.bin", {"e4": 60, "d4": 30, "c4": 10})
        answer = server.book_moves(book=str(book))
        assert answer["entries_in_book"] == 3
        assert answer["count"] == 3
        assert answer["moves"] == [
            {"san": "e4", "uci": "e2e4", "weight": 60, "share": 0.6, "learn": 0},
            {"san": "d4", "uci": "d2d4", "weight": 30, "share": 0.3, "learn": 0},
            {"san": "c4", "uci": "c2c4", "weight": 10, "share": 0.1, "learn": 0},
        ]
        assert answer["best"] == "e4"

    def test_a_limit_keeps_the_heaviest_moves(self, tmp_path):
        book = written_book(tmp_path / "three.bin", {"e4": 60, "d4": 30, "c4": 10})
        answer = server.book_moves(book=str(book), limit=2)
        assert [move["san"] for move in answer["moves"]] == ["e4", "d4"]
        assert answer["count"] == 3

    def test_a_position_the_book_does_not_hold_answers_with_nothing(self, tmp_path):
        book = written_book(tmp_path / "one.bin", {"e4": 7})
        answer = server.book_moves(book=str(book), moves=["e4", "e5"])
        assert answer["count"] == 0
        assert answer["moves"] == []
        assert answer["best"] is None


class TestPgn:
    def test_a_game_is_read_into_its_parts(self):
        answer = server.pgn(text=positions.FOOLS_MATE_PGN)
        assert answer["headers"] == {"Event": "Test", "Site": "?", "White": "A", "Black": "B", "Result": "0-1"}
        assert list(answer["headers"]) == ["Event", "Site", "White", "Black", "Result"]
        assert [move["san"] for move in answer["moves"]] == positions.FOOLS_MATE
        assert [move["uci"] for move in answer["moves"]] == ["f2f3", "e7e5", "g2g4", "d8h4"]
        assert [move["comment_after"] for move in answer["moves"]] == ["", "", "a blunder", ""]
        assert answer["status"]["state"] == "checkmate"
        assert answer["opening"]["name"].endswith("Fool's Mate")

    def test_a_malformed_source_is_an_error_object(self):
        answer = server.pgn(text="[Event")
        assert answer["error"]["kind"] == "invalid_pgn"

    def test_asking_for_a_game_that_is_not_there_says_how_many_there_are(self):
        answer = server.pgn(text=positions.FOOLS_MATE_PGN, index=3)
        assert answer["error"]["kind"] == "no_such_game"
        assert answer["error"]["games_in_source"] == 1

    def test_a_position_read_from_a_pgn_is_the_one_the_game_ended_on(self):
        read = server.pgn(text=positions.FOOLS_MATE_PGN)
        played = server.position(pgn=positions.FOOLS_MATE_PGN)
        assert played["fen"] == read["final_fen"]
        assert played["status"]["state"] == "checkmate"


class TestToPgn:
    def test_moves_come_back_as_a_game(self):
        answer = server.to_pgn(moves=positions.FOOLS_MATE, headers={"White": "A", "Black": "B"})
        assert "1. f3 e5 2. g4 Qh4# 0-1" in answer["pgn"]
        assert '[White "A"]' in answer["pgn"]
        assert answer["result"] == "0-1"

    def test_what_is_written_reads_back_the_same(self):
        written = server.to_pgn(moves=["e4", "c5", "Nf3"])
        assert server.pgn(text=written["pgn"])["final_fen"] == written["final_fen"]

    def test_an_illegal_move_names_its_place_in_the_sequence(self):
        answer = server.to_pgn(moves=["e4", "e4"])
        assert answer["error"]["kind"] == "illegal_move"
        assert answer["error"]["move_index"] == 1
        assert "e5" in answer["error"]["legal_moves"]


class TestCatalogueAndPrompt:
    def test_the_catalogue_names_every_group_the_facts_tool_serves(self):
        catalogue = server.facts_catalogue()
        served = server.facts(groups=None)["groups_available"]
        assert [group["name"] for group in catalogue["groups"]] == served

    @pytest.mark.parametrize("group", GROUPS, ids=GROUPS)
    def test_every_field_of_a_group_is_named_typed_and_explained(self, group):
        catalogue = server.facts_catalogue()
        answered = server.facts(groups=[group])["groups"][group]
        [described] = [entry for entry in catalogue["groups"] if entry["name"] == group]
        assert [field["name"] for field in described["fields"]] == list(answered)
        assert [field["name"] for field in described["fields"] if not field["type"]] == []
        assert [field["name"] for field in described["fields"] if not field["meaning"].endswith(".")] == []

    def test_the_catalogue_says_how_to_read_every_shape_a_value_takes(self):
        types = server.facts_catalogue()["types"]
        assert set(types) == {"ByColour", "SquareSet", "FileSet", "PerFile", "PerRank", "[T; n]", "Option"}
        assert "file a first" in types["PerFile"]

    def test_the_catalogue_covers_a_moves_facts_too(self):
        catalogue = server.facts_catalogue()
        names = [field["name"] for field in catalogue["move"]]
        assert "see" in names
        assert "gives_check" in names
        assert catalogue["types"]["ByColour"]

    def test_the_prompt_names_the_position_and_the_tools(self):
        text = server.analyse_position(fen=positions.START, moves="e4 e5")
        assert positions.START in text
        assert "e4 e5" in text
        assert "legal_moves" in text
