"""Each tool called in process, on positions derived by hand."""

import esca
import pytest

from chess_esca_mcp import server
from tests import positions


def reasons(answer: dict) -> set[str]:
    """The reason names an illegal move came back with."""
    return {item["reason"] for item in answer["reasons"]}


def obstacles(answer: dict, side: str, wing: str) -> set[str]:
    """The reason names standing in the way of one castling."""
    return {item["reason"] for item in answer["castling"][side][wing]["obstacles"]}


class TestPosition:
    def test_start_position_is_read_whole(self):
        answer = server.position()
        assert answer["fen"] == positions.START
        assert answer["side_to_move"] == "white"
        assert answer["legal_move_count"] == 20
        assert answer["status"]["state"] == "in_progress"
        assert answer["check"]["in_check"] is False
        assert answer["material"]["white"]["pawn"] == 8
        assert answer["material"]["balance"] == 0

    def test_moves_are_played_from_the_position(self):
        answer = server.position(moves=["e4", "e5", "Nf3"])
        assert answer["ply"] == 3
        assert answer["line"] == ["e4", "e5", "Nf3"]
        assert answer["side_to_move"] == "black"
        assert answer["opening"] == {"eco": "C40", "name": "King's Knight Opening"}

    @pytest.mark.parametrize(
        ("wing", "expected"),
        [("short", {"path_attacked"}), ("long", set())],
    )
    def test_an_attacked_path_stops_one_castling_only(self, wing, expected):
        answer = server.position(fen=positions.CASTLING_ATTACKED)
        assert obstacles(answer, "white", wing) == expected

    def test_every_blocked_castling_names_its_occupants(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)
        assert obstacles(answer, "white", "short") == {"path_blocked"}
        blocked = answer["castling"]["white"]["short"]["obstacles"][0]
        assert [unit["square"] for unit in blocked["occupants"]] == ["f1"]
        assert answer["castling"]["white"]["short"]["king_lands_on"] == "g1"

    def test_an_en_passant_offer_lists_the_pawns_beside_it(self):
        answer = server.position(fen=positions.EN_PASSANT)["en_passant"]
        assert answer["target"] == "d6"
        assert answer["available"] is True
        assert [capture["origin"] for capture in answer["captures"]] == ["e5"]

    def test_an_en_passant_capture_that_exposes_the_king_says_so(self):
        answer = server.position(fen=positions.EN_PASSANT_EXPOSES_KING)["en_passant"]
        assert answer["available"] is False
        forbidden = answer["captures"][0]["forbidden_by"]
        assert forbidden["reason"] == "exposes_king"
        assert forbidden["attacker"] == "h5"

    def test_fools_mate_is_a_finished_game(self):
        answer = server.position(moves=positions.FOOLS_MATE)
        assert answer["status"]["state"] == "checkmate"
        assert answer["status"]["winner"] == "black"
        assert answer["status"]["result"] == "0-1"
        assert answer["legal_move_count"] == 0
        assert [unit["square"] for unit in answer["check"]["checkers"]] == ["h4"]

    def test_a_threefold_repetition_is_claimable_with_its_plies(self):
        answer = server.position(moves=positions.THREEFOLD)["status"]
        assert answer["claims"] == ["threefold_repetition"]
        claimable = answer["claimable"][0]
        assert claimable["kind"] == "threefold"
        assert claimable["repetition"]["count"] == 3
        assert claimable["repetition"]["plies"] == [0, 4, 8]
        assert claimable["repetition"]["near_misses"] == []

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
    )
    def test_an_automatic_draw_is_named_and_evidenced(self, fen, kind):
        answer = server.position(fen=fen)["status"]
        assert answer["state"] == kind
        assert answer["result"] == "1/2-1/2"
        assert [draw["kind"] for draw in answer["automatic"]] == [kind]

    def test_the_stalemate_evidence_names_every_square_the_king_is_denied(self):
        answer = server.position(fen=positions.STALEMATE)["status"]["automatic"][0]
        assert answer["stalemate"]["king"] == "h8"
        denied = {square["square"] for square in answer["stalemate"]["escape_squares"]}
        assert denied == {"g7", "g8", "h7"}

    @pytest.mark.parametrize(
        ("fen", "kind"),
        [(positions.NOT_A_FEN, "invalid_fen"), (positions.NO_KINGS, "invalid_fen")],
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
        assert {move["san"] for move in answer["moves"]} >= {"e4", "Nf3", "a3"}
        assert answer["by_category"]["quiet"] == sorted(answer["by_category"]["quiet"])

    def test_categories_hold_what_they_are_named_for(self):
        answer = server.legal_moves(fen=positions.CASTLING_ATTACKED)
        assert answer["by_category"]["castling"] == ["O-O-O"]
        assert set(answer["by_category"]["checks"]) == {"Rxa8+", "Rxh8+"}
        assert set(answer["by_category"]["captures"]) == {"Rxa8+", "Rxh8+"}

    def test_categories_can_be_left_out(self):
        assert "by_category" not in server.legal_moves(categories=False)

    def test_an_en_passant_capture_is_listed_as_one(self):
        answer = server.legal_moves(fen=positions.EN_PASSANT)
        assert answer["by_category"]["en_passant"] == ["exd6"]
        capture = next(move for move in answer["moves"] if move["san"] == "exd6")
        assert capture["kind"] == "en_passant"
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

    def test_an_impossible_move_names_the_unit_and_what_it_could_play(self):
        answer = server.explain_move(move="e2e5")
        assert answer["legal"] is False
        assert reasons(answer) == {"not_a_move_of_this_unit"}
        assert answer["legal_moves_from_origin"] == ["e3", "e4"]

    def test_an_empty_origin_is_named_as_one(self):
        answer = server.explain_move(move="e4e5")
        assert reasons(answer) == {"empty_origin"}

    def test_a_move_of_the_other_side_is_named_as_one(self):
        answer = server.explain_move(move="e7e5")
        assert reasons(answer) == {"not_the_side_to_move"}

    def test_a_forbidden_castling_answers_with_every_obstacle_at_once(self):
        answer = server.explain_move(move="O-O", fen=positions.CASTLING_ATTACKED)
        assert reasons(answer) == {"path_attacked"}
        attacked = answer["reasons"][0]["squares"][0]
        assert attacked["square"] == "f1"
        assert [unit["square"] for unit in attacked["attackers"]] == ["b5"]

    def test_a_castling_without_the_right_says_so(self):
        answer = server.explain_move(move="O-O", fen="r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1")
        assert reasons(answer) == {"no_castling_right"}

    @pytest.mark.parametrize("move", ["e5d6", "exd6"])
    def test_an_en_passant_capture_that_exposes_the_king_says_which_unit_waits(self, move):
        answer = server.explain_move(move=move, fen=positions.EN_PASSANT_EXPOSES_KING)
        assert answer["legal"] is False
        obstacle = next(item for item in answer["reasons"] if item.get("capture") == "en_passant")
        assert obstacle["reason"] == "exposes_king"
        assert obstacle["attacker"] == "h5"

    def test_a_promotion_left_unnamed_is_named_as_the_reason(self):
        answer = server.explain_move(move="a7a8", fen="8/P6k/8/8/8/8/8/K7 w - - 0 1")
        assert "promotion_not_named" in reasons(answer)

    def test_text_that_is_no_move_at_all_says_so(self):
        answer = server.explain_move(move="zzz")
        assert reasons(answer) == {"unreadable_move_text"}

    def test_a_bad_position_is_an_error_before_the_move_is_read(self):
        assert server.explain_move(move="e4", fen=positions.NOT_A_FEN)["error"]["kind"] == "invalid_fen"


class TestFacts:
    def test_the_default_leaves_out_the_two_raw_groups(self):
        answer = server.facts()
        assert "placement" not in answer["groups"]
        assert "planes" not in answer["groups"]
        assert set(answer["groups_available"]) > set(answer["groups_returned"])

    def test_only_the_named_groups_come_back(self):
        answer = server.facts(groups=["material", "state"])
        assert set(answer["groups"]) == {"material", "state"}
        assert answer["schema_id"]

    def test_every_side_paired_value_is_labelled_by_colour(self):
        answer = server.facts(groups=["material"])["groups"]["material"]
        assert answer["count"]["white"]["pawn"] == 8
        assert answer["value"] == {"white": 39, "black": 39}

    def test_the_perspective_says_which_side_the_facts_are_read_from(self):
        white = server.facts()["perspective"]
        black = server.facts(moves=["e4"])["perspective"]
        assert white["us"] == "white"
        assert black["us"] == "black"

    def test_a_role_indexed_list_is_labelled_by_role(self):
        answer = server.facts(groups=["placement"])["groups"]["placement"]
        assert answer["by_role"]["white"]["king"] == ["e1"]
        assert answer["by_role"]["black"]["pawn"] == ["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"]

    def test_a_file_indexed_list_is_labelled_by_file(self):
        answer = server.facts(groups=["pawns"])["groups"]["pawns"]
        assert answer["count_by_file"]["white"] == dict.fromkeys("abcdefgh", 1)

    def test_the_king_shield_is_labelled_by_the_files_it_stands_on(self):
        answer = server.facts(groups=["king"])["groups"]["king"]
        assert answer["shield_files"]["white"] == "def"
        assert set(answer["shield"]["white"]) == set("def")

    def test_an_unknown_group_lists_the_known_ones(self):
        answer = server.facts(groups=["nope"])
        assert answer["error"]["kind"] == "unknown_fact_group"
        assert "material" in answer["error"]["groups_available"]

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
        assert answer["count"]["white"]["rook"] == 1
        assert answer["count"]["black"] == dict.fromkeys(answer["count"]["black"], 0) | {"king": 1}
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
        assert answer["prose"]

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
    def test_a_blocked_castling_says_in_words_what_blocks_it(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)
        said = answer["castling"]["white"]["short"]["prose"]
        assert "f1" in said[0]
        assert said[0] in answer["prose"]

    def test_the_gathered_sentences_repeat_none_of_themselves(self):
        answer = server.position(fen=positions.CASTLING_BLOCKED)
        assert len(answer["prose"]) == len(set(answer["prose"]))
        assert answer["en_passant"]["prose"][0] in answer["prose"]

    def test_a_pin_says_in_words_what_holds_the_unit(self):
        answer = server.position(fen=positions.PIN_ON_THE_BACK_RANK)
        pinned = answer["pins"]["white"][0]
        assert "g1" in pinned["prose"][0]
        assert pinned["prose"][0] in answer["prose"]

    def test_the_draw_status_speaks_once_for_all_the_conditions_under_it(self):
        answer = server.position(fen=positions.STALEMATE)["status"]
        assert answer["prose"]
        assert "prose" not in answer["automatic"][0]
        assert answer["automatic"][0]["stalemate"]["prose"]

    def test_an_illegal_castling_is_explained_in_words(self):
        answer = server.explain_move(move="O-O", fen=positions.CASTLING_ATTACKED)
        assert answer["legal"] is False
        assert "f1" in answer["prose"][0]

    def test_an_en_passant_obstacle_names_the_waiting_unit_in_words(self):
        answer = server.explain_move(move="e5d6", fen=positions.EN_PASSANT_EXPOSES_KING)
        obstacle = next(item for item in answer["reasons"] if item.get("capture") == "en_passant")
        assert "h5" in obstacle["prose"][0]
        assert obstacle["prose"][0] in answer["prose"]

    def test_a_legal_move_carries_the_sentences_of_what_it_leaves_behind(self):
        answer = server.explain_move(move="Ng8", moves=positions.THREEFOLD[:7])
        assert answer["claims_after"][0]["prose"][0] in answer["prose"]


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
        book = tmp_path / "one.bin"
        game = esca.Game()
        entry = esca.polyglot.Entry(game.position.polyglot_key, game.legal_moves()[0], 7, 0)
        esca.polyglot.Book.write(book, [entry])
        answer = server.book_moves(book=str(book))
        assert answer["entries_in_book"] == 1
        assert answer["count"] == 1
        assert answer["moves"][0]["weight"] == 7
        assert answer["best"] == answer["moves"][0]["san"]

    def test_a_position_the_book_does_not_hold_answers_with_nothing(self, tmp_path):
        book = tmp_path / "one.bin"
        game = esca.Game()
        esca.polyglot.Book.write(book, [esca.polyglot.Entry(game.position.polyglot_key, game.legal_moves()[0])])
        answer = server.book_moves(book=str(book), moves=["e4", "e5"])
        assert answer["count"] == 0
        assert answer["best"] is None


class TestPgn:
    def test_a_game_is_read_into_its_parts(self):
        answer = server.pgn(text=positions.FOOLS_MATE_PGN)
        assert answer["headers"]["White"] == "A"
        assert [move["san"] for move in answer["moves"]] == positions.FOOLS_MATE
        assert answer["moves"][2]["comment_after"] == "a blunder"
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


class TestSchemaAndPrompt:
    def test_the_facts_schema_names_every_group_the_facts_tool_serves(self):
        schema = server.facts_schema()
        served = server.facts(groups=None)["groups_available"]
        assert [group["name"] for group in schema["groups"]] == served
        assert schema["schema_id"] == server.facts()["schema_id"]

    def test_every_group_of_the_schema_carries_its_feature_names(self):
        schema = server.facts_schema()
        assert sum(len(group["features"]) for group in schema["groups"]) == schema["feature_count"]

    def test_the_prompt_names_the_position_and_the_tools(self):
        text = server.analyse_position(fen=positions.START, moves="e4 e5")
        assert positions.START in text
        assert "e4 e5" in text
        assert "legal_moves" in text
