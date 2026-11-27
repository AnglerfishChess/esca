//! The typed tensor form: one array per fact, batch first, nothing scaled.

mod common;

use esca::facts::tensors::{Batch, Buffer, Entry, Form, layout};
use esca::facts::types::{Dtype, Packing};
use esca::{AnnotatedMove, CHESS960, CLASSIC, MoveFacts, Position, Square, Variant};
use rstest::rstest;
use serde_json::Value;

/// One position with a white passer on d5 and nothing but kings and pawns.
const PASSER: &str = "8/pp3ppk/8/3P4/8/8/PP3PPK/8 w - - 0 1";
/// The opening array.
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// A middlegame busy with attacks, pins and castling rights.
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
/// White may take on f6 en passant.
const EN_PASSANT: &str = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
/// Both sides have a pawn one step from promotion.
const PROMOTIONS: &str = "8/1P4k1/8/8/8/8/6p1/K7 w - - 0 1";
/// Black to move with all four castlings open.
const CASTLINGS: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
/// Bishop and knight against a bare king.
const BISHOP_AND_KNIGHT: &str = "8/8/8/4k3/8/8/8/4KBN1 w - - 0 1";
/// One bishop each, on opposite square colours.
const OPPOSITE_BISHOPS: &str = "8/5k2/8/3b4/8/8/2B5/4K3 w - - 0 1";
/// White is mated.
const FOOLS_MATE: &str = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
/// Kings in direct opposition.
const OPPOSITION: &str = "8/8/4k3/8/4K3/8/8/8 w - - 0 1";
/// A Chess960 start, where the back rank is not the classic one.
const CHESS960_START: &str = "bqnbnrkr/pppppppp/8/8/8/8/PPPPPPPP/BQNBNRKR w HFhf - 0 1";

/// The classic positions the agreement tests read.
const VARIED: [&str; 10] = [
    START,
    PASSER,
    KIWIPETE,
    EN_PASSANT,
    PROMOTIONS,
    CASTLINGS,
    BISHOP_AND_KNIGHT,
    OPPOSITE_BISHOPS,
    FOOLS_MATE,
    OPPOSITION,
];

fn batch_of(fens: &[&str]) -> Batch {
    let positions: Vec<Position> = fens
        .iter()
        .map(|fen| Position::from_fen(fen).expect("a test FEN is a legal position"))
        .collect();
    Batch::from_positions(&positions, &CLASSIC)
}

fn moves_of(fens: &[&str]) -> Vec<AnnotatedMove> {
    fens.iter()
        .flat_map(|fen| common::position_of(fen).annotated_moves(&CLASSIC).to_vec())
        .collect()
}

fn names(entries: &[Entry]) -> Vec<String> {
    entries.iter().map(|entry| entry.name.to_string()).collect()
}

#[test]
fn every_fact_of_every_group_has_a_layout_entry() {
    let mut positions: Vec<String> = Vec::new();
    for group in esca::catalogue() {
        positions.extend(
            group
                .fields
                .iter()
                .map(|field| format!("{}.{}", group.name, field.name)),
        );
    }
    let moves: Vec<String> = MoveFacts::FIELDS
        .iter()
        .map(|field| format!("move.{}", field.name))
        .collect();
    assert_eq!(names(layout().positions()), positions);
    assert_eq!(names(layout().moves()), moves);
    assert_eq!(
        names(&layout().entries),
        [positions, moves].concat(),
        "the position groups come first, then the move group"
    );
}

#[test]
fn every_array_holds_what_the_layout_says() {
    let batch = batch_of(&[START, PASSER, KIWIPETE]);
    assert_eq!(batch.len(), 3);
    assert_eq!(batch.form(), Form::Expanded);
    assert_eq!(batch.entries(), layout().positions());
    for (entry, buffer) in batch.entries().iter().zip(batch.buffers()) {
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
    for entry in batch.entries() {
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

#[test]
fn every_move_array_holds_what_the_layout_says() {
    let moves = moves_of(&[START, KIWIPETE]);
    // 20 legal moves at the start, 48 in Kiwipete.
    assert_eq!(moves.len(), 68);
    let batch = Batch::from_moves(&moves);
    assert_eq!(batch.len(), 68);
    assert_eq!(batch.entries(), layout().moves());
    for (entry, buffer) in batch.entries().iter().zip(batch.buffers()) {
        assert_eq!(buffer.dtype(), entry.dtype, "{}", entry.name);
        assert_eq!(buffer.len(), batch.len() * entry.width(), "{}", entry.name);
        assert_eq!(batch.shape_of(entry), vec![68], "{}", entry.name);
    }
}

#[rstest]
#[case::a_flag("move.gives_check", Dtype::Bool)]
#[case::a_value("move.see", Dtype::Int(32))]
#[case::a_named_value("move.mover", Dtype::Uint(8))]
#[case::an_absent_value("move.victim", Dtype::Int(16))]
fn a_move_fact_keeps_the_type_it_was_declared_with(#[case] name: &str, #[case] dtype: Dtype) {
    let entry = layout().entry(name).expect("a fact of that name");
    assert_eq!(entry.dtype, dtype);
    assert_eq!(entry.shape, &[] as &[usize]);
}

#[test]
fn a_move_batch_writes_each_move_in_turn() {
    // Black to move, the g2 pawn one step from promotion.
    let promotions = "8/1P4k1/8/8/8/8/6p1/K7 b - - 0 1";
    let facts: Vec<MoveFacts> = ["g2g1q", "g2g1n", "g7f6"]
        .iter()
        .map(|uci| common::move_facts(promotions, uci))
        .collect();
    let batch = Batch::from_move_facts(&facts);
    let Buffer::U8(movers) = batch.buffer("move.mover").expect("the movers") else {
        panic!("a role is an unsigned byte")
    };
    // p = 0, k = 5.
    assert_eq!(movers, &[0, 0, 5]);
    let Buffer::I16(promotions) = batch.buffer("move.promotion").expect("the promotions") else {
        panic!("an optional role is a signed 16-bit value")
    };
    // q = 4, n = 1, and the king move promotes nothing.
    assert_eq!(promotions, &[4, 1, -1]);
    let Buffer::I16(victims) = batch.buffer("move.victim").expect("the victims") else {
        panic!("an optional role is a signed 16-bit value")
    };
    assert_eq!(victims, &[-1, -1, -1]);
}

#[test]
fn the_packed_move_form_holds_the_same_facts() {
    let batch = Batch::from_moves(&moves_of(&[START, KIWIPETE, PASSER]));
    let packed = batch.packed();
    assert_eq!(packed.entries(), layout().moves());
    assert_eq!(packed.expanded(), batch);
}

#[test]
fn the_golden_move_batch_is_what_the_moves_answer() {
    let batch = Batch::from_moves(&moves_of(&[
        START,
        &START.replace(" w ", " b "),
        KIWIPETE,
        PASSER,
    ]));
    assert_eq!(
        batch.to_safetensors(),
        include_bytes!("data/facts/moves.safetensors"),
        "tests/data/facts/moves.safetensors is stale: run `cargo run --example catalogue --all-features`"
    );
    assert_eq!(
        batch.packed().to_safetensors(),
        include_bytes!("data/facts/moves.packed.safetensors"),
        "tests/data/facts/moves.packed.safetensors is stale: run `cargo run --example catalogue --all-features`"
    );
}

/// One value of the JSON form, as the values its tensor holds: the reading of
/// `docs/features.md` the native filler has to agree with.
fn walked(value: &Value, kind: &Value, out: &mut Vec<f64>) {
    let codes = |kind: &Value| -> Vec<String> {
        kind["codes"]
            .as_array()
            .expect("the codes of a named value")
            .iter()
            .map(|code| code.as_str().expect("a code").to_string())
            .collect()
    };
    let code_of = |kind: &Value, value: &Value| -> f64 {
        let name = value.as_str().expect("a name");
        codes(kind)
            .iter()
            .position(|code| code == name)
            .expect("a known name") as f64
    };
    let flags = |names: &[String], value: &Value, out: &mut Vec<f64>| {
        let held: Vec<&str> = value
            .as_array()
            .expect("a list of names")
            .iter()
            .map(|name| name.as_str().expect("a name"))
            .collect();
        out.extend(
            names
                .iter()
                .map(|name| f64::from(u8::from(held.contains(&name.as_str())))),
        );
    };
    match kind["kind"].as_str().expect("a kind") {
        "by_colour" => {
            walked(&value["white"], &kind["item"], out);
            walked(&value["black"], &kind["item"], out);
        }
        "fixed" => {
            for item in value.as_array().expect("a run") {
                walked(item, &kind["item"], out);
            }
        }
        "square_set" => {
            let names: Vec<String> = Square::ALL.iter().map(Square::to_string).collect();
            flags(&names, value, out);
        }
        "file_set" => {
            let names: Vec<String> = "abcdefgh".chars().map(String::from).collect();
            flags(&names, value, out);
        }
        "optional" if value.is_null() => out.push(-1.0),
        "optional" if kind["item"]["kind"] == "named" => out.push(code_of(&kind["item"], value)),
        "named" => out.push(code_of(kind, value)),
        "bool" => out.push(f64::from(u8::from(value.as_bool().expect("a flag")))),
        _ => out.push(value.as_f64().expect("a number")),
    }
}

/// The bytes the JSON form of `items` gives the array of `entry`.
fn walked_bytes(items: &[Value], entry: &Entry) -> Vec<u8> {
    let kind = &entry.to_json()["kind"];
    let mut values = Vec::new();
    for item in items {
        walked(&item[entry.group][entry.field], kind, &mut values);
    }
    values
        .into_iter()
        .flat_map(|value| match entry.dtype {
            Dtype::Bool | Dtype::Uint(8) => vec![value as u8],
            Dtype::Int(8) => vec![value as i8 as u8],
            Dtype::Uint(16) => (value as u16).to_le_bytes().to_vec(),
            Dtype::Int(16) => (value as i16).to_le_bytes().to_vec(),
            Dtype::Uint(32) => (value as u32).to_le_bytes().to_vec(),
            Dtype::Int(32) => (value as i32).to_le_bytes().to_vec(),
            Dtype::Float32 => (value as f32).to_le_bytes().to_vec(),
            other => panic!("no fact is {other}"),
        })
        .collect()
}

fn agrees_with_the_json_form(batch: &Batch, items: &[Value]) {
    for (entry, buffer) in batch.entries().iter().zip(batch.buffers()) {
        assert_eq!(
            buffer.bytes(),
            walked_bytes(items, entry),
            "{} disagrees with its JSON form",
            entry.name
        );
    }
}

#[rstest]
#[case::classic(&CLASSIC, &VARIED)]
#[case::chess960(&CHESS960, &[CHESS960_START, START])]
fn every_position_array_agrees_with_the_json_form(
    #[case] variant: &dyn Variant,
    #[case] fens: &[&str],
) {
    let positions: Vec<Position> = fens.iter().map(|fen| common::position_of(fen)).collect();
    let batch = Batch::from_positions(&positions, variant);
    let items: Vec<Value> = positions
        .iter()
        .map(|position| serde_json::to_value(position.facts(variant)).expect("the facts serialise"))
        .collect();
    agrees_with_the_json_form(&batch, &items);
}

#[test]
fn every_move_array_agrees_with_the_json_form() {
    let moves = moves_of(&VARIED);
    let batch = Batch::from_moves(&moves);
    let items: Vec<Value> = moves
        .iter()
        .map(|annotated| {
            serde_json::json!({ "move": serde_json::to_value(annotated.facts).expect("the facts serialise") })
        })
        .collect();
    agrees_with_the_json_form(&batch, &items);
}
