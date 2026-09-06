//! Move generation against published perft counts.
//!
//! Classic positions 1-6 are the standard set from the Chess Programming
//! Wiki's "Perft Results" page; the Chess960 positions are the standard
//! Scharnagl-numbered perft suite. Each position's FEN and counts are given
//! in full below.

use esca::{CHESS960, CLASSIC, MoveList, Position, Variant};

fn perft(variant: &dyn Variant, position: &Position, depth: u32) -> u64 {
    let mut moves = MoveList::new();
    variant.legal_moves(position, &mut moves);
    if depth <= 1 {
        return moves.len() as u64;
    }
    moves
        .as_slice()
        .iter()
        .map(|&mv| perft(variant, &variant.play(position, mv), depth - 1))
        .sum()
}

fn check(variant: &dyn Variant, fen: &str, counts: &[u64]) {
    let position = Position::from_fen(fen).expect("the FEN is well formed");
    for (index, &expected) in counts.iter().enumerate() {
        let depth = index as u32 + 1;
        assert_eq!(
            perft(variant, &position, depth),
            expected,
            "{fen} at depth {depth}"
        );
    }
}

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

const C960_333: &str = "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9";
const C960_404: &str = "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9";
const C960_789: &str = "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9";
const C960_726: &str = "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9";

#[test]
fn classic_startpos() {
    check(&CLASSIC, STARTPOS, &[20, 400, 8902, 197_281, 4_865_609]);
}

#[test]
fn classic_kiwipete() {
    check(&CLASSIC, KIWIPETE, &[48, 2039, 97_862, 4_085_603]);
}

#[test]
fn classic_position_3() {
    check(&CLASSIC, POSITION_3, &[14, 191, 2812, 43_238, 674_624]);
}

#[test]
fn classic_position_4() {
    check(&CLASSIC, POSITION_4, &[6, 264, 9467, 422_333]);
}

#[test]
fn classic_position_5() {
    check(&CLASSIC, POSITION_5, &[44, 1486, 62_379, 2_103_487]);
}

#[test]
fn classic_position_6() {
    check(&CLASSIC, POSITION_6, &[46, 2079, 89_890, 3_894_594]);
}

#[test]
fn chess960_position_333() {
    check(&CHESS960, C960_333, &[29, 502, 14_569, 287_739]);
}

#[test]
fn chess960_position_404() {
    check(&CHESS960, C960_404, &[27, 916, 25_798, 890_435]);
}

#[test]
fn chess960_position_789() {
    check(&CHESS960, C960_789, &[24, 600, 15_347, 408_207]);
}

#[test]
fn chess960_position_726() {
    check(&CHESS960, C960_726, &[29, 861, 24_504, 763_454]);
}

/// Arrangement 518 is the classic array, so Chess960 must count it the way
/// classic chess does.
#[test]
fn chess960_arrangement_518_is_classic() {
    let start = CHESS960.start_position(518);
    assert_eq!(start.fen(), STARTPOS);
    check(&CHESS960, &start.fen(), &[20, 400, 8902, 197_281]);
}

#[test]
#[ignore = "slow: several million nodes per position"]
fn deep_counts() {
    check(
        &CLASSIC,
        STARTPOS,
        &[20, 400, 8902, 197_281, 4_865_609, 119_060_324],
    );
    check(
        &CLASSIC,
        KIWIPETE,
        &[48, 2039, 97_862, 4_085_603, 193_690_690],
    );
    check(
        &CLASSIC,
        POSITION_3,
        &[14, 191, 2812, 43_238, 674_624, 11_030_083],
    );
    check(&CLASSIC, POSITION_4, &[6, 264, 9467, 422_333, 15_833_292]);
    check(
        &CLASSIC,
        POSITION_5,
        &[44, 1486, 62_379, 2_103_487, 89_941_194],
    );
    check(
        &CLASSIC,
        POSITION_6,
        &[46, 2079, 89_890, 3_894_594, 164_075_551],
    );
    check(
        &CHESS960,
        C960_333,
        &[29, 502, 14_569, 287_739, 8_652_810, 191_762_235],
    );
    check(
        &CHESS960,
        C960_404,
        &[27, 916, 25_798, 890_435, 26_302_461, 924_181_432],
    );
    check(
        &CHESS960,
        C960_789,
        &[24, 600, 15_347, 408_207, 11_029_596, 308_553_169],
    );
    check(
        &CHESS960,
        C960_726,
        &[29, 861, 24_504, 763_454, 22_763_215, 731_511_256],
    );
}
