//! FEN and EPD reading and writing.

use esca::{CHESS960, Colour, FenError, File, Position, Square, Variant};

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[test]
fn round_trips() {
    for fen in [
        STARTPOS,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 137 200",
    ] {
        let position = Position::from_fen(fen).expect("the FEN is well formed");
        assert_eq!(position.fen(), fen);
        assert!(position.clocks_known());
    }
}

#[test]
fn epd_takes_default_clocks_and_marks_them_unknown() {
    let position = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -")
        .expect("a four-field FEN is an EPD");
    assert!(!position.clocks_known());
    assert_eq!(position.halfmove_clock(), 0);
    assert_eq!(position.fullmove_number(), 1);
    assert_eq!(position.fen(), STARTPOS);
    assert_eq!(
        position.epd(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
    );
}

/// The same Chess960 position written both ways: `KQkq` naming the outermost
/// rooks (X-FEN) and the rook files themselves (Shredder-FEN).
#[test]
fn xfen_and_shredder_castling_agree() {
    const SHREDDER: &str = "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9";
    const XFEN: &str = "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w KQkq - 0 9";

    let shredder = Position::from_fen(SHREDDER).expect("the FEN is well formed");
    let xfen = Position::from_fen(XFEN).expect("the FEN is well formed");
    assert_eq!(shredder, xfen);

    let rights = shredder.castling_rights();
    assert_eq!(rights.short(Colour::White), Some(File::F));
    assert_eq!(rights.long(Colour::White), Some(File::B));
    assert_eq!(rights.short(Colour::Black), Some(File::F));
    assert_eq!(rights.long(Colour::Black), Some(File::B));

    // Shuffled rook files leave no classic spelling, so both write Shredder.
    assert_eq!(shredder.fen(), SHREDDER);
    assert_eq!(xfen.fen(), SHREDDER);
}

/// Rooks on the classic files are written `KQkq` even when the position came
/// from Shredder text and even under Chess960.
#[test]
fn classic_rook_files_are_written_kqkq() {
    let position = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1")
        .expect("the FEN is well formed");
    assert_eq!(position.fen(), STARTPOS);
    assert_eq!(CHESS960.start_position(518).fen(), STARTPOS);
}

#[test]
fn partial_castling_rights() {
    for fen in [
        "r3k2r/8/8/8/8/8/8/R3K2R w K - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w Qk - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1",
    ] {
        assert_eq!(
            Position::from_fen(fen)
                .expect("the FEN is well formed")
                .fen(),
            fen
        );
    }
}

#[test]
fn every_chess960_start_position_round_trips() {
    for arrangement in 0..960 {
        let start = CHESS960.start_position(arrangement);
        let fen = start.fen();
        assert_eq!(
            Position::from_fen(&fen).expect("the FEN is well formed"),
            start,
            "arrangement {arrangement}"
        );
    }
}

#[test]
fn en_passant_square_is_read_and_written() {
    let position =
        Position::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2")
            .expect("the FEN is well formed");
    assert_eq!(position.en_passant(), Some(Square::E6));
}

#[test]
fn rejects_malformed_text() {
    for (fen, expected) in [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq",
            FenError::FieldCount,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1",
            FenError::Placement,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
            FenError::SideToMove,
        ),
        ("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1", FenError::Castling),
        ("4k3/8/8/8/8/8/8/4K3 w - e4 0 1", FenError::EnPassant),
        ("4k3/8/8/8/8/8/8/4K3 w - - x 1", FenError::HalfmoveClock),
        ("4k3/8/8/8/8/8/8/4K3 w - - 0 0", FenError::FullmoveNumber),
        ("4k3/8/8/8/8/8/8/8 w - - 0 1", FenError::Position),
    ] {
        assert_eq!(Position::from_fen(fen), Err(expected), "{fen}");
    }
}

#[test]
fn mirroring_swaps_colours_and_flips_ranks() {
    let position =
        Position::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2")
            .expect("the FEN is well formed");
    let mirrored = position.mirrored();
    assert_eq!(mirrored.side_to_move(), Colour::Black);
    assert_eq!(mirrored.en_passant(), Some(Square::E3));
    assert_eq!(mirrored.mirrored(), position);
}
