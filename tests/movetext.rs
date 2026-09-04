//! UCI and SAN text, in both directions.

use esca::{
    CHESS960, CLASSIC, CastlingOutput, Game, Move, MoveList, MoveParseError, Position, Variant,
    chess960, classic,
};

fn legal_moves(variant: &dyn Variant, position: &Position) -> MoveList {
    let mut moves = MoveList::new();
    variant.legal_moves(position, &mut moves);
    moves
}

fn round_trip_all(variant: &dyn Variant, fen: &str) {
    let position = Position::from_fen(fen).expect("the FEN is well formed");
    for &mv in legal_moves(variant, &position).as_slice() {
        for style in [CastlingOutput::KingToRook, CastlingOutput::KingTwoSquares] {
            let uci = variant.move_to_uci(&position, mv, style);
            assert_eq!(
                variant.move_from_uci(&position, &uci),
                Ok(mv),
                "{fen}: uci {uci}"
            );
        }
        let san = variant.move_to_san(&position, mv);
        assert_eq!(
            variant.move_from_san(&position, &san),
            Ok(mv),
            "{fen}: san {san}"
        );
    }
}

#[test]
fn classic_text_round_trips() {
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1",
        "1n1n4/2P1P3/8/8/4k3/8/8/4K3 w - - 0 1",
    ] {
        round_trip_all(&CLASSIC, fen);
    }
}

#[test]
fn chess960_text_round_trips() {
    for fen in [
        "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9",
        "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9",
        "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9",
        "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9",
    ] {
        round_trip_all(&CHESS960, fen);
    }
}

#[test]
fn classic_castling_is_written_in_the_style_asked_for() {
    let position =
        Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("the FEN is well formed");
    let short = CLASSIC
        .move_from_san(&position, "O-O")
        .expect("white may castle short");
    let long = CLASSIC
        .move_from_san(&position, "O-O-O")
        .expect("white may castle long");

    assert_eq!(
        CLASSIC.move_to_uci(&position, short, CastlingOutput::KingToRook),
        "e1h1"
    );
    assert_eq!(
        CLASSIC.move_to_uci(&position, short, CastlingOutput::KingTwoSquares),
        "e1g1"
    );
    assert_eq!(
        CLASSIC.move_to_uci(&position, long, CastlingOutput::KingToRook),
        "e1a1"
    );
    assert_eq!(
        CLASSIC.move_to_uci(&position, long, CastlingOutput::KingTwoSquares),
        "e1c1"
    );

    // Both spellings are read whatever the output style is set to.
    for text in ["e1g1", "e1h1"] {
        assert_eq!(CLASSIC.move_from_uci(&position, text), Ok(short), "{text}");
    }
    for text in ["e1c1", "e1a1"] {
        assert_eq!(CLASSIC.move_from_uci(&position, text), Ok(long), "{text}");
    }
    assert_eq!(CLASSIC.move_to_san(&position, short), "O-O");
    assert_eq!(CLASSIC.move_to_san(&position, long), "O-O-O");
}

#[test]
fn chess960_castling_is_always_king_to_rook() {
    // The king starts on b1 and its only rook on a1; castling long still
    // lands the king on c1 and the rook on d1.
    let position =
        Position::from_fen("rk6/8/8/8/8/8/8/RK6 w Aa - 0 1").expect("the FEN is well formed");
    let long = CHESS960
        .move_from_san(&position, "O-O-O")
        .expect("white may castle long");
    for style in [CastlingOutput::KingToRook, CastlingOutput::KingTwoSquares] {
        assert_eq!(CHESS960.move_to_uci(&position, long, style), "b1a1");
    }
    // Black's remaining right names the a-file rook, so it writes as `q`.
    assert_eq!(
        CHESS960.play(&position, long).fen(),
        "rk6/8/8/8/8/8/8/2KR4 b q - 1 1"
    );
}

#[test]
fn game_uci_output_follows_its_castling_style() {
    let mut game = Game::from_fen(classic(), "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
        .expect("the FEN is well formed");
    let short = CLASSIC
        .move_from_san(game.position(), "O-O")
        .expect("white may castle short");
    assert_eq!(game.castling_output(), CastlingOutput::KingToRook);
    assert_eq!(game.move_to_uci(short), "e1h1");
    game.set_castling_output(CastlingOutput::KingTwoSquares);
    assert_eq!(game.move_to_uci(short), "e1g1");
}

#[test]
fn san_disambiguates_by_file_then_rank_then_both() {
    // Four queens bear on d4: from a4 only the rank tells them apart, from
    // a1 neither file nor rank does, and d1 and g1 have files of their own.
    let position =
        Position::from_fen("1k6/8/8/8/Q7/8/8/Q2QK1Q1 w - - 0 1").expect("the FEN is well formed");
    let mut text: Vec<String> = legal_moves(&CLASSIC, &position)
        .as_slice()
        .iter()
        .filter(|mv| mv.to() == esca::Square::D4)
        .map(|&mv| CLASSIC.move_to_san(&position, mv))
        .collect();
    text.sort();
    assert_eq!(text, ["Q4d4", "Qa1d4", "Qdd4", "Qgd4"]);
}

#[test]
fn san_reads_the_forms_it_does_not_write() {
    let position = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("the FEN is well formed");
    let knight = CLASSIC
        .move_from_san(&position, "Nf3")
        .expect("the knight may go to f3");
    for text in ["Nf3", "Ngf3", "N1f3", "Ng1f3", "Ng1-f3!?"] {
        assert_eq!(CLASSIC.move_from_san(&position, text), Ok(knight), "{text}");
    }
}

#[test]
fn san_marks_check_and_mate() {
    // After 1. f3 e5 2. g4, Black mates.
    let position =
        Position::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2")
            .expect("the FEN is well formed");
    let mate = CLASSIC
        .move_from_uci(&position, "d8h4")
        .expect("the queen may go to h4");
    assert_eq!(CLASSIC.move_to_san(&position, mate), "Qh4#");

    let position =
        Position::from_fen("4k3/8/8/8/8/8/8/4KQ2 w - - 0 1").expect("the FEN is well formed");
    let check = CLASSIC
        .move_from_uci(&position, "f1f8")
        .expect("the queen may go to f8");
    assert_eq!(CLASSIC.move_to_san(&position, check), "Qf8+");
}

#[test]
fn promotion_text() {
    let position = Position::from_fen("1n1n4/2P1P3/8/8/4k3/8/8/4K3 w - - 0 1")
        .expect("the FEN is well formed");
    let capture = CLASSIC
        .move_from_uci(&position, "c7b8q")
        .expect("the pawn may take and promote");
    assert_eq!(CLASSIC.move_to_san(&position, capture), "cxb8=Q");
    assert_eq!(
        CLASSIC.move_to_uci(&position, capture, CastlingOutput::KingToRook),
        "c7b8q"
    );
    assert!(capture.is_capture());
    assert_eq!(capture.promotion(), Some(esca::Role::Queen));
    assert_eq!(CLASSIC.move_from_san(&position, "cxb8=Q"), Ok(capture));
}

#[test]
fn en_passant_text() {
    let position =
        Position::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
            .expect("the FEN is well formed");
    let mv = CLASSIC
        .move_from_san(&position, "exf6")
        .expect("the pawn may take en passant");
    assert!(mv.is_en_passant());
    assert!(mv.is_capture());
    assert_eq!(
        CLASSIC.move_to_uci(&position, mv, CastlingOutput::KingToRook),
        "e5f6"
    );
}

#[test]
fn rejects_text_that_names_no_move() {
    let position = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("the FEN is well formed");
    assert_eq!(
        CLASSIC.move_from_uci(&position, "e2e5"),
        Err(MoveParseError::Illegal)
    );
    assert_eq!(
        CLASSIC.move_from_uci(&position, "xyzzy"),
        Err(MoveParseError::Syntax)
    );
    assert_eq!(
        CLASSIC.move_from_san(&position, "O-O"),
        Err(MoveParseError::Illegal)
    );
    assert_eq!(
        CLASSIC.move_from_san(&position, "Qz9"),
        Err(MoveParseError::Syntax)
    );
}

#[test]
fn san_reports_ambiguity() {
    let position =
        Position::from_fen("R7/8/8/4k3/8/8/8/R3K3 w - - 0 1").expect("the FEN is well formed");
    assert_eq!(
        CLASSIC.move_from_san(&position, "Ra5"),
        Err(MoveParseError::Ambiguous)
    );
}

#[test]
fn a_game_plays_text_of_either_notation() {
    let mut game = Game::new(classic());
    for text in ["e4", "e5", "Nf3", "Nc6", "Bb5"] {
        game.play_san(text).expect("the moves are legal");
    }
    assert_eq!(
        game.position().fen(),
        "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3"
    );
    let played: Vec<String> = game.moves().iter().map(Move::to_string).collect();
    assert_eq!(played, ["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"]);

    let mut game = Game::with_seed(chess960(), 518);
    game.play_uci("e2e4").expect("the move is legal");
    assert_eq!(game.ply(), 1);
}
