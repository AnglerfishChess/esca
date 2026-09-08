//! The typed tensor form: one array per fact, batch first, nothing scaled.

mod common;

use esca::facts::tensors::{Batch, Buffer, Form, layout};
use esca::facts::types::{Dtype, Packing};
use esca::{CLASSIC, Position, Square};
use rstest::rstest;

/// One position with a white passer on d5 and nothing but kings and pawns.
const PASSER: &str = "8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1";
/// The opening array.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// A middlegame busy with attacks, pins and castling rights.
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

fn batch_of(fens: &[&str]) -> Batch {
    let positions: Vec<Position> = fens
        .iter()
        .map(|fen| Position::from_fen(fen).expect("a test FEN is a legal position"))
        .collect();
    Batch::from_positions(&positions, &CLASSIC)
}

#[test]
fn the_layout_names_every_fact_of_every_group() {
    let mut named: Vec<String> = Vec::new();
    for group in esca::catalogue() {
        named.extend(
            group
                .fields
                .iter()
                .map(|field| format!("{}.{}", group.name, field.name)),
        );
    }
    let entries: Vec<String> = layout()
        .entries
        .iter()
        .map(|entry| entry.name.to_string())
        .collect();
    assert_eq!(entries, named);
}

#[test]
fn every_array_holds_what_the_layout_says() {
    let batch = batch_of(&[START, PASSER, KIWIPETE]);
    assert_eq!(batch.len(), 3);
    assert_eq!(batch.form(), Form::Expanded);
    for (entry, buffer) in layout().entries.iter().zip(batch.buffers()) {
        assert_eq!(
            buffer.dtype(),
            entry.dtype,
            "{} holds the wrong element type",
            entry.name
        );
        assert_eq!(
            buffer.len(),
            batch.len() * entry.width(),
            "{} holds the wrong number of values",
            entry.name
        );
        assert_eq!(
            batch.shape_of(entry)[0],
            batch.len(),
            "{} is not batch-first",
            entry.name
        );
    }
}

#[rstest]
#[case::a_flag("state.in_check", Dtype::Bool, &[])]
#[case::a_flag_per_side("state.can_castle_short", Dtype::Bool, &[2])]
#[case::a_count("pawns.count_by_file", Dtype::Uint(8), &[2, 8])]
#[case::a_value("material.value", Dtype::Int(32), &[2])]
#[case::a_fraction("king.tropism", Dtype::Float32, &[2])]
#[case::an_absent_value("pawns.passer_lead_rank", Dtype::Int(16), &[2])]
#[case::a_square_set("pawns.passed", Dtype::Bool, &[2, 64])]
#[case::a_file_set("pawns.open_files", Dtype::Bool, &[8])]
#[case::a_named_value("king.square", Dtype::Uint(8), &[2])]
#[case::a_run_of_three("king.shield", Dtype::Int(16), &[2, 3])]
fn a_fact_keeps_the_type_it_was_declared_with(
    #[case] name: &str,
    #[case] dtype: Dtype,
    #[case] shape: &[usize],
) {
    let entry = layout().entry(name).expect("a fact of that name");
    assert_eq!(entry.dtype, dtype);
    assert_eq!(entry.shape, shape);
}

#[test]
fn a_square_set_is_one_flag_per_square_a1_first() {
    let batch = batch_of(&[PASSER]);
    let entry = layout().entry("pawns.passed").expect("the passers");
    let Buffer::Bool(flags) = batch.buffer("pawns.passed").expect("the passers") else {
        panic!("a square set is a run of flags")
    };
    // White's plane comes first, then Black's; d5 is White's only passer and
    // Black has none.
    assert_eq!(flags.len(), 2 * 64);
    let white: Vec<usize> = (0..64).filter(|&index| flags[index]).collect();
    let black: Vec<usize> = (0..64).filter(|&index| flags[64 + index]).collect();
    assert_eq!(white, vec![Square::D5.index()]);
    assert!(black.is_empty());
    assert_eq!(
        entry.order,
        "axis 1: white, black; axis 2: the 64 squares, a1 first, then b1 through h1, then a2"
    );
}

#[test]
fn a_count_stands_where_its_file_says() {
    let batch = batch_of(&[PASSER]);
    let Buffer::U8(counts) = batch
        .buffer("pawns.count_by_file")
        .expect("the counts by file")
    else {
        panic!("a count is an unsigned byte")
    };
    // White's pawns stand on a2 b2 d5 f2 g2, Black's on a7 b7 f7 g7.
    assert_eq!(&counts[..8], &[1, 1, 0, 1, 0, 1, 1, 0]);
    assert_eq!(&counts[8..], &[1, 1, 0, 0, 0, 1, 1, 0]);
}

#[test]
fn an_absent_value_is_minus_one() {
    let batch = batch_of(&[PASSER]);
    let Buffer::I16(ranks) = batch
        .buffer("pawns.passer_lead_rank")
        .expect("the lead ranks")
    else {
        panic!("an optional count is a signed 16-bit value")
    };
    // White's passer stands on its rank 5; Black has no passer at all.
    assert_eq!(ranks, &[5, -1]);
}

#[test]
fn a_named_value_is_its_code() {
    let batch = batch_of(&[PASSER]);
    let Buffer::U8(squares) = batch.buffer("king.square").expect("the kings") else {
        panic!("a square is an unsigned byte")
    };
    assert_eq!(
        squares,
        &[Square::H2.index() as u8, Square::H7.index() as u8]
    );
}

#[test]
fn nothing_is_a_float_but_the_one_fact_that_is_fractional() {
    let floats: Vec<&str> = layout()
        .entries
        .iter()
        .filter(|entry| entry.dtype == Dtype::Float32)
        .map(|entry| entry.name)
        .collect();
    assert_eq!(floats, vec!["king.tropism"]);
}

#[test]
fn the_packed_form_holds_the_same_facts() {
    let batch = batch_of(&[START, PASSER, KIWIPETE]);
    let packed = batch.packed();
    assert_eq!(packed.form(), Form::Packed);
    assert_eq!(packed.expanded(), batch);
    assert_eq!(packed.packed(), packed);
    assert_eq!(batch.expanded(), batch);
}

#[test]
fn a_packed_square_set_is_one_bitboard() {
    let batch = batch_of(&[PASSER]).packed();
    let Buffer::U64(boards) = batch.buffer("pawns.passed").expect("the passers") else {
        panic!("a packed square set is a bitboard")
    };
    assert_eq!(boards, &[1u64 << Square::D5.index(), 0]);

    let Buffer::U8(masks) = batch.buffer("pawns.open_files").expect("the open files") else {
        panic!("a packed file set is a byte mask")
    };
    // Files c, e and h carry no pawn of either colour.
    assert_eq!(masks, &[0b1001_0100]);
}

#[test]
fn a_packed_flag_is_one_bit() {
    let batch = batch_of(&[START, PASSER, KIWIPETE]);
    let entry = layout()
        .entry("material.pawns_only")
        .expect("the pawns-only flag");
    assert_eq!(entry.packing(), Packing::Bits);
    let packed = batch.packed();
    let Buffer::U8(bits) = packed
        .buffer("material.pawns_only")
        .expect("the pawns-only flag")
    else {
        panic!("a packed flag is a byte of bits")
    };
    // Only the passer position has nothing but kings and pawns, and it is the
    // second of the three: the top bit of the byte is the first position.
    assert_eq!(bits, &[0b0100_0000]);
}

#[test]
fn the_safetensors_header_names_every_array() {
    let batch = batch_of(&[START, PASSER]);
    let bytes = batch.to_safetensors();
    let length = u64::from_le_bytes(bytes[..8].try_into().expect("a header length")) as usize;
    let header = std::str::from_utf8(&bytes[8..8 + length]).expect("a JSON header");
    assert!(header.starts_with("{\"__metadata__\":"));
    assert!(header.contains("\"form\":\"expanded\""));
    assert!(header.contains("\"count\":\"2\""));
    for entry in &layout().entries {
        assert!(
            header.contains(&format!("\"{}\":", entry.name)),
            "{} is missing from the header",
            entry.name
        );
    }
    assert!(header.contains("\"pawns.passed\":{\"dtype\":\"BOOL\",\"shape\":[2,2,64]"));
}

#[test]
fn the_golden_batch_is_what_the_positions_answer() {
    // The classic positions of `tests/data/facts/positions.json`, in the order
    // that file names them; the Chess960 one is left out because one batch is
    // read under one set of rules.
    let batch = batch_of(&[START, &START.replace(" w ", " b "), KIWIPETE, PASSER]);
    assert_eq!(
        batch.to_safetensors(),
        include_bytes!("data/facts/batch.safetensors"),
        "tests/data/facts/batch.safetensors is stale: run `cargo run --example catalogue --all-features`"
    );
    assert_eq!(
        batch.packed().to_safetensors(),
        include_bytes!("data/facts/batch.packed.safetensors"),
        "tests/data/facts/batch.packed.safetensors is stale: run `cargo run --example catalogue --all-features`"
    );
}
