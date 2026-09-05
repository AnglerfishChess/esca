//! Named endings: what material is left, which ending of the books it is,
//! what theory says the result is, and the method that gets it.
//!
//! [`classify`] answers a position with one [`Ending`]. The class and the
//! verdict are the general case — what the books say about this material —
//! and the position-specific facts that can overturn them are grouped in
//! [`Evidence`], each behind the reason it belongs to.

use crate::explain::{colour_word, english_list};
use crate::facts::scan::{between, distance};
use crate::game::Game;
use crate::position::Position;
use crate::types::{Colour, File, Rank, Role, Square, SquareSet};

/// The conventional value of a role: pawn 1, knight and bishop 3, rook 5,
/// queen 9, king 0.
const fn value(role: Role) -> u32 {
    match role {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 0,
    }
}

/// The roles a material signature writes, in the order it writes them: the
/// king, then the pieces by descending value, then the pawns.
const SIGNATURE_ORDER: [Role; 6] = [
    Role::King,
    Role::Queen,
    Role::Rook,
    Role::Bishop,
    Role::Knight,
    Role::Pawn,
];

/// The most pieces one side may have for the position to be an ending. A
/// piece is any unit that is neither a king nor a pawn.
pub const ENDING_PIECES: u32 = 2;

/// The material both sides hold, written the way endings are named.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Signature {
    /// The canonical spelling, stronger side first: `KRPvKR`.
    pub text: String,
    /// The side written first: the one with more conventional material; on a
    /// tie the one whose pieces are worth more one by one, queen before rook
    /// before bishop before knight before pawn; White when even that is even.
    pub stronger: Colour,
    /// Units per role per colour, indexed by [`Colour::index`] and
    /// [`Role::index`].
    pub counts: [[u8; 6]; 2],
    /// Conventional material per colour, the king counting nothing.
    pub value: [u32; 2],
}

impl Signature {
    /// How many units of `role` `colour` has.
    pub fn count(&self, colour: Colour, role: Role) -> u8 {
        self.counts[colour.index()][role.index()]
    }

    /// The pieces of `colour`: everything that is neither a king nor a pawn.
    pub fn pieces(&self, colour: Colour) -> u32 {
        [Role::Queen, Role::Rook, Role::Bishop, Role::Knight]
            .into_iter()
            .map(|role| u32::from(self.count(colour, role)))
            .sum()
    }

    /// The pawns of both sides.
    pub fn pawns(&self) -> u32 {
        Colour::ALL
            .into_iter()
            .map(|colour| u32::from(self.count(colour, Role::Pawn)))
            .sum()
    }

    /// One plain sentence naming what each side has.
    pub fn describe(&self) -> String {
        format!(
            "The material is {}: White has {}, Black has {}.",
            self.text,
            self.material_of(Colour::White),
            self.material_of(Colour::Black),
        )
    }

    /// What `colour` has besides its king, in words.
    fn material_of(&self, colour: Colour) -> String {
        let items: Vec<String> = [
            Role::Queen,
            Role::Rook,
            Role::Bishop,
            Role::Knight,
            Role::Pawn,
        ]
        .into_iter()
        .filter(|&role| self.count(colour, role) > 0)
        .map(|role| units(self.count(colour, role), role))
        .collect();
        if items.is_empty() {
            return "nothing besides its king".to_string();
        }
        english_list(&items)
    }
}

/// `count` units of `role`: "a rook", "two bishops".
fn units(count: u8, role: Role) -> String {
    const WORDS: [&str; 9] = [
        "no", "a", "two", "three", "four", "five", "six", "seven", "eight",
    ];
    let name = match role {
        Role::Pawn => "pawn",
        Role::Knight => "knight",
        Role::Bishop => "bishop",
        Role::Rook => "rook",
        Role::Queen => "queen",
        Role::King => "king",
    };
    let many = usize::from(count) >= WORDS.len();
    let word = if many {
        count.to_string()
    } else {
        WORDS[usize::from(count)].to_string()
    };
    if count == 1 {
        format!("{word} {name}")
    } else {
        format!("{word} {name}s")
    }
}

/// The named endings this version of the catalogue tells apart. A class names
/// material only: which side holds which half of it is [`Verdict`]'s answer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    /// Kings only.
    KvK,
    /// Queen against a lone king.
    KQvK,
    /// Rook against a lone king.
    KRvK,
    /// Two bishops against a lone king.
    KBBvK,
    /// Bishop and knight against a lone king.
    KBNvK,
    /// Two knights against a lone king.
    KNNvK,
    /// One bishop against a lone king.
    KBvK,
    /// One knight against a lone king.
    KNvK,
    /// One pawn against a lone king.
    KPvK,
    /// Bishop and one pawn against a lone king.
    KBPvK,
    /// A queen each.
    KQvKQ,
    /// Queen against rook.
    KQvKR,
    /// Queen against bishop.
    KQvKB,
    /// Queen against knight.
    KQvKN,
    /// Queen against one pawn.
    KQvKP,
    /// Queen against two minor pieces.
    KQvTwoMinors,
    /// A rook each.
    KRvKR,
    /// Rook against bishop.
    KRvKB,
    /// Rook against knight.
    KRvKN,
    /// Rook against one pawn.
    KRvKP,
    /// Rook against two minor pieces.
    KRvTwoMinors,
    /// Rook and one pawn against rook.
    KRPvKR,
    /// Two bishops against knight.
    KBBvKN,
    /// Bishop against knight.
    KBvKN,
    /// A bishop each, both on one square colour.
    KBvKBSameColour,
    /// A bishop each, on opposite square colours.
    KBvKBOppositeColour,
    /// A knight each.
    KNvKN,
    /// Bishop against one pawn.
    KBvKP,
    /// Knight against one pawn.
    KNvKP,
    /// Kings and pawns only, in any number.
    Pawns,
    /// An ending the catalogue does not name.
    Other,
    /// More material than [`ENDING_PIECES`] allows: not an ending at all.
    NotAnEnding,
}

impl Class {
    /// Every class, in the order this catalogue lists them.
    pub const ALL: [Class; 32] = [
        Class::KvK,
        Class::KQvK,
        Class::KRvK,
        Class::KBBvK,
        Class::KBNvK,
        Class::KNNvK,
        Class::KBvK,
        Class::KNvK,
        Class::KPvK,
        Class::KBPvK,
        Class::KQvKQ,
        Class::KQvKR,
        Class::KQvKB,
        Class::KQvKN,
        Class::KQvKP,
        Class::KQvTwoMinors,
        Class::KRvKR,
        Class::KRvKB,
        Class::KRvKN,
        Class::KRvKP,
        Class::KRvTwoMinors,
        Class::KRPvKR,
        Class::KBBvKN,
        Class::KBvKN,
        Class::KBvKBSameColour,
        Class::KBvKBOppositeColour,
        Class::KNvKN,
        Class::KBvKP,
        Class::KNvKP,
        Class::Pawns,
        Class::Other,
        Class::NotAnEnding,
    ];

    /// The name in `snake_case`, as the Python surface spells it.
    pub fn name(self) -> &'static str {
        match self {
            Class::KvK => "k_v_k",
            Class::KQvK => "kq_v_k",
            Class::KRvK => "kr_v_k",
            Class::KBBvK => "kbb_v_k",
            Class::KBNvK => "kbn_v_k",
            Class::KNNvK => "knn_v_k",
            Class::KBvK => "kb_v_k",
            Class::KNvK => "kn_v_k",
            Class::KPvK => "kp_v_k",
            Class::KBPvK => "kbp_v_k",
            Class::KQvKQ => "kq_v_kq",
            Class::KQvKR => "kq_v_kr",
            Class::KQvKB => "kq_v_kb",
            Class::KQvKN => "kq_v_kn",
            Class::KQvKP => "kq_v_kp",
            Class::KQvTwoMinors => "kq_v_two_minors",
            Class::KRvKR => "kr_v_kr",
            Class::KRvKB => "kr_v_kb",
            Class::KRvKN => "kr_v_kn",
            Class::KRvKP => "kr_v_kp",
            Class::KRvTwoMinors => "kr_v_two_minors",
            Class::KRPvKR => "krp_v_kr",
            Class::KBBvKN => "kbb_v_kn",
            Class::KBvKN => "kb_v_kn",
            Class::KBvKBSameColour => "kb_v_kb_same_colour",
            Class::KBvKBOppositeColour => "kb_v_kb_opposite_colour",
            Class::KNvKN => "kn_v_kn",
            Class::KBvKP => "kb_v_kp",
            Class::KNvKP => "kn_v_kp",
            Class::Pawns => "pawns",
            Class::Other => "other",
            Class::NotAnEnding => "not_an_ending",
        }
    }

    /// One plain sentence naming the ending.
    pub fn describe(self) -> &'static str {
        match self {
            Class::KvK => "Only the two kings are left, so nothing can be won.",
            Class::KQvK => {
                "King and queen against a lone king: the queen takes squares away until the \
                 lone king stands on the edge, and the king comes up to mate."
            }
            Class::KRvK => {
                "King and rook against a lone king: the rook cuts the lone king off and the two \
                 push it to the edge."
            }
            Class::KBBvK => {
                "King and two bishops against a lone king: the two diagonals build a wall that \
                 forces the lone king into a corner of either colour."
            }
            Class::KBNvK => {
                "King, bishop and knight against a lone king: the longest of the basic mates, \
                 and only in a corner the bishop covers."
            }
            Class::KNNvK => {
                "King and two knights against a lone king: mate can be reached on the board but \
                 never forced."
            }
            Class::KBvK => {
                "King and one bishop against a lone king, which no sequence of legal moves can \
                 ever mate."
            }
            Class::KNvK => {
                "King and one knight against a lone king, which no sequence of legal moves can \
                 ever mate."
            }
            Class::KPvK => {
                "King and pawn against a lone king, the ending every pawn ending comes down to."
            }
            Class::KBPvK => {
                "King, bishop and pawn against a lone king: the extra piece wins unless the \
                 bishop is of the wrong colour for a rook pawn."
            }
            Class::KQvKQ => {
                "A queen each and nothing else: neither king can be sheltered from perpetual \
                 check."
            }
            Class::KQvKR => {
                "Queen against rook: a win, but one that takes a long and exact technique."
            }
            Class::KQvKB => "Queen against a lone bishop, which the queen wins easily.",
            Class::KQvKN => "Queen against a lone knight, which the queen wins easily.",
            Class::KQvKP => {
                "Queen against a lone pawn: won, except where a rook or bishop pawn on the \
                 seventh rank buys a stalemate."
            }
            Class::KQvTwoMinors => {
                "Queen against two minor pieces, which defend each other well enough that the \
                 position decides the result."
            }
            Class::KRvKR => "A rook each and nothing else, which neither side can win.",
            Class::KRvKB => {
                "Rook against a lone bishop: a draw once the bishop's king reaches a safe corner."
            }
            Class::KRvKN => {
                "Rook against a lone knight: a draw as long as knight and king stay together."
            }
            Class::KRvKP => {
                "Rook against a lone pawn: the rook wins once it gets behind the pawn in time."
            }
            Class::KRvTwoMinors => {
                "Rook against two minor pieces, where the two pieces are the better side without \
                 a forced win."
            }
            Class::KRPvKR => {
                "Rook and pawn against rook, the most common ending of all: the Lucena position \
                 wins it and the Philidor position holds it."
            }
            Class::KBBvKN => {
                "Two bishops against a knight: a win, but one that can take a hundred moves."
            }
            Class::KBvKN => "Bishop against knight and nothing else, which neither side can win.",
            Class::KBvKBSameColour => {
                "A bishop each on the same square colour, so the two bishops meet but nothing \
                 can be forced."
            }
            Class::KBvKBOppositeColour => {
                "A bishop each on opposite square colours, so the two bishops never meet."
            }
            Class::KNvKN => "A knight each and nothing else, which neither side can win.",
            Class::KBvKP => {
                "Bishop against a lone pawn: the bishop draws wherever it can reach the pawn's \
                 path."
            }
            Class::KNvKP => {
                "Knight against a lone pawn: the knight draws wherever it can reach the pawn's \
                 path in time."
            }
            Class::Pawns => {
                "A pawn ending: kings and pawns only, where the opposition and the race decide \
                 everything."
            }
            Class::Other => {
                "An ending this catalogue does not name; the material signature says what is on \
                 the board."
            }
            Class::NotAnEnding => "Too much material is left for this to be an ending at all.",
        }
    }
}

/// What theory says the result of an ending is, played out by both sides at
/// their best. It is the answer for the ending, not a search of the position
/// in hand.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Verdict {
    /// A forced win for that colour.
    Win(Colour),
    /// Usually won by that colour; particular positions are drawn.
    UsuallyWin(Colour),
    /// Usually drawn; the positions that are not are won by that colour.
    UsuallyDraw(Colour),
    /// A draw against correct defence.
    Draw,
    /// Theory gives no single result.
    Unknown,
}

impl Verdict {
    /// The colour the verdict names, if it names one.
    pub fn winner(self) -> Option<Colour> {
        match self {
            Verdict::Win(colour) | Verdict::UsuallyWin(colour) | Verdict::UsuallyDraw(colour) => {
                Some(colour)
            }
            Verdict::Draw | Verdict::Unknown => None,
        }
    }

    /// The name in `snake_case`, as the Python surface spells it.
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Win(_) => "win",
            Verdict::UsuallyWin(_) => "usually_win",
            Verdict::UsuallyDraw(_) => "usually_draw",
            Verdict::Draw => "draw",
            Verdict::Unknown => "unknown",
        }
    }

    /// One plain sentence naming the result.
    pub fn describe(self) -> String {
        match self {
            Verdict::Win(colour) => format!(
                "{} wins this ending by force, against any defence.",
                colour_word(colour)
            ),
            Verdict::UsuallyWin(colour) => format!(
                "{} usually wins this ending, though particular positions are drawn.",
                colour_word(colour)
            ),
            Verdict::UsuallyDraw(colour) => format!(
                "This ending is usually drawn, and the positions that are not are won by {}.",
                colour_word(colour)
            ),
            Verdict::Draw => {
                "This ending is a draw against correct defence, whoever is to move.".to_string()
            }
            Verdict::Unknown => {
                "Theory gives this ending no single result; it has to be judged position by \
                 position."
                    .to_string()
            }
        }
    }
}

/// The named method an ending is played by.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Technique {
    /// The ending has no named method.
    None,
    /// The box the lone king moves in is made smaller move by move.
    BoxMethod,
    /// The two bishops drive the lone king to a corner.
    TwoBishopMate,
    /// Bishop and knight drive the lone king to a corner the bishop covers.
    BishopKnightMate,
    /// The squares in front of a pawn that win it the game.
    KeySquares,
    /// The kings face each other and the one not to move gains ground.
    Opposition,
    /// Whether a lone king catches a running pawn, read off the board.
    RuleOfTheSquare,
    /// The rook shelters its own king from checks and the pawn queens.
    Lucena,
    /// The rook holds the third rank and then checks from behind.
    Philidor,
    /// The bishop does not cover the square the pawn promotes on.
    WrongBishop,
    /// A rook pawn whose promotion corner the lone king has reached.
    WrongRookPawn,
}

impl Technique {
    /// Every technique, in the order this catalogue lists them.
    pub const ALL: [Technique; 11] = [
        Technique::None,
        Technique::BoxMethod,
        Technique::TwoBishopMate,
        Technique::BishopKnightMate,
        Technique::KeySquares,
        Technique::Opposition,
        Technique::RuleOfTheSquare,
        Technique::Lucena,
        Technique::Philidor,
        Technique::WrongBishop,
        Technique::WrongRookPawn,
    ];

    /// The name in `snake_case`, as the Python surface spells it.
    pub fn name(self) -> &'static str {
        match self {
            Technique::None => "none",
            Technique::BoxMethod => "box_method",
            Technique::TwoBishopMate => "two_bishop_mate",
            Technique::BishopKnightMate => "bishop_knight_mate",
            Technique::KeySquares => "key_squares",
            Technique::Opposition => "opposition",
            Technique::RuleOfTheSquare => "rule_of_the_square",
            Technique::Lucena => "lucena",
            Technique::Philidor => "philidor",
            Technique::WrongBishop => "wrong_bishop",
            Technique::WrongRookPawn => "wrong_rook_pawn",
        }
    }

    /// One plain sentence naming the method and how it is played.
    pub fn describe(self) -> &'static str {
        match self {
            Technique::None => "This ending is played by no method with a name of its own.",
            Technique::BoxMethod => {
                "The box method: the piece draws a box the lone king may not leave, and every \
                 move makes the box one file or one rank smaller until the king stands on the \
                 edge."
            }
            Technique::TwoBishopMate => {
                "The two bishops stand side by side on next-door diagonals, which the lone king \
                 cannot cross, and the king walks it into a corner."
            }
            Technique::BishopKnightMate => {
                "The lone king is driven to a corner of the bishop's own colour, the knight \
                 walking the W-shaped path along the edge that closes off the escape."
            }
            Technique::KeySquares => {
                "The pawn queens if its own king reaches one of the key squares in front of it, \
                 which is what the defending king has to be kept off."
            }
            Technique::Opposition => {
                "The kings stand on one line with an odd number of squares between them; the \
                 side not to move holds the opposition, and the other has to give ground."
            }
            Technique::RuleOfTheSquare => {
                "Draw the square whose side is the pawn's run to promotion: a lone king that \
                 cannot step inside it never catches the pawn."
            }
            Technique::Lucena => {
                "The Lucena position: with the pawn on the seventh and its own king in front of \
                 it, the rook builds a bridge that shelters the king from the checks, and the \
                 pawn queens."
            }
            Technique::Philidor => {
                "The Philidor position: the defending rook holds its own third rank until the \
                 pawn steps onto it, then drops to the far end of the board and checks the \
                 attacking king from behind."
            }
            Technique::WrongBishop => {
                "The bishop stands on the colour the pawn does not promote on, so the defending \
                 king sits on the promotion square and can never be driven off it."
            }
            Technique::WrongRookPawn => {
                "The pawn is a rook pawn and the lone king has reached its promotion corner, \
                 where the attacking king cannot come near without stalemating it."
            }
        }
    }
}

/// The race of the only pawn on the board. The defending side is the one
/// without the pawn.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PawnRace {
    /// Where the pawn stands.
    pub pawn: Square,
    /// Whose pawn it is.
    pub colour: Colour,
    /// The square it promotes on.
    pub promotion: Square,
    /// It stands on the a- or the h-file.
    pub rook_pawn: bool,
    /// Pawn moves left to promotion, a double first step counted as one.
    pub steps: u32,
    /// The defending king reaches the promotion square no later than the pawn
    /// does, both racing straight there and neither king counted as an
    /// obstacle: the rule of the square.
    pub defender_inside_square: bool,
    /// The pawn's own king stands on the pawn's file, ahead of the pawn.
    pub attacker_in_front: bool,
    /// The defending king stands on the pawn's file, ahead of the pawn.
    pub defender_in_front: bool,
    /// The defending king stands on the promotion square or beside it.
    pub defender_holds_the_corner: bool,
}

impl PawnRace {
    /// One plain sentence: whose pawn is running where, and whether the other
    /// king catches it.
    pub fn describe(&self) -> String {
        format!(
            "{}'s pawn on {} is {} moves from promoting on {}, and the defending king {} the \
             square of the pawn.",
            colour_word(self.colour),
            self.pawn,
            self.steps,
            self.promotion,
            if self.defender_inside_square {
                "is inside"
            } else {
                "is outside"
            },
        )
    }
}

/// The bishops on the board, when at least one stands on it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Bishops {
    /// Each side has exactly one bishop and the two stand on opposite square
    /// colours, so neither ever attacks the other's squares.
    pub opposite_colours: bool,
    /// Every bishop on the board stands on one square colour.
    pub same_colour: bool,
    /// The only pawn on the board belongs to the side with the bishops, is a
    /// rook pawn, and no bishop of that side stands on the colour of the
    /// square it promotes on.
    pub wrong_bishop: bool,
}

impl Bishops {
    /// One plain sentence naming what the bishops can and cannot reach.
    pub fn describe(&self) -> String {
        if self.wrong_bishop {
            return "The bishop stands on the square colour its rook pawn does not promote on, \
                    so it can never cover the promotion square."
                .to_string();
        }
        if self.opposite_colours {
            return "The two bishops stand on opposite square colours and can never attack each \
                    other."
                .to_string();
        }
        if self.same_colour {
            return "Every bishop on the board stands on one square colour.".to_string();
        }
        "The bishops on the board cover both square colours.".to_string()
    }
}

/// The position-specific facts an ending's verdict and technique are read
/// off. Each group is present only when the material puts it in question.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Evidence {
    /// The race, when exactly one pawn stands on the board.
    pub pawn: Option<PawnRace>,
    /// The bishops, when at least one stands on the board.
    pub bishops: Option<Bishops>,
    /// The kings stand on one file, rank or diagonal with exactly one empty
    /// square between them, so the side not to move holds the opposition.
    pub opposition: bool,
}

impl Evidence {
    /// One plain sentence per group that applies, and a sentence saying so
    /// when none does.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(pawn) = &self.pawn {
            parts.push(pawn.describe());
        }
        if let Some(bishops) = &self.bishops {
            parts.push(bishops.describe());
        }
        if self.opposition {
            parts.push(
                "The kings stand in opposition, so the side not to move has to be given way to."
                    .to_string(),
            );
        }
        if parts.is_empty() {
            return "Nothing in this position changes what theory says about the ending."
                .to_string();
        }
        parts.join(" ")
    }
}

/// What ending a position is, and what is known about it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ending {
    /// The named ending, [`Class::Other`] when the catalogue has no name for
    /// it and [`Class::NotAnEnding`] when there is too much material.
    pub class: Class,
    /// The material both sides hold, whatever the class.
    pub signature: Signature,
    /// The result, theory's answer adjusted by [`Ending::evidence`].
    pub verdict: Verdict,
    /// The method the ending is played by.
    pub technique: Technique,
    /// The position-specific facts behind `verdict` and `technique`.
    pub evidence: Evidence,
}

impl Ending {
    /// The material, the ending, the result and the method, in plain
    /// sentences.
    pub fn describe(&self) -> String {
        if self.class == Class::NotAnEnding {
            return format!("{} {}", self.signature.describe(), self.class.describe());
        }
        let mut parts = vec![
            self.signature.describe(),
            self.class.describe().to_string(),
            self.verdict.describe(),
        ];
        if self.technique != Technique::None {
            parts.push(self.technique.describe().to_string());
        }
        parts.join(" ")
    }
}

impl Position {
    /// What ending this position is. See [`classify`].
    pub fn ending(&self) -> Ending {
        classify(self)
    }
}

impl Game {
    /// What ending the current position is. See [`classify`].
    pub fn ending(&self) -> Ending {
        classify(self.position())
    }
}

/// The ending `position` is: its material signature, the named class, the
/// result theory gives that class, the method it is played by, and the
/// position-specific facts those were read off.
///
/// A position is an ending when neither side has more than [`ENDING_PIECES`]
/// pieces, a piece being any unit that is neither a king nor a pawn. Above
/// that the class is [`Class::NotAnEnding`], and the signature still says
/// what is on the board.
pub fn classify(position: &Position) -> Ending {
    let signature = signature(position);
    let evidence = evidence(position, &signature);
    let class = class_of(&signature, &evidence);
    let (verdict, technique) = judge(class, &signature, &evidence);
    Ending {
        class,
        signature,
        verdict,
        technique,
        evidence,
    }
}

/// The material of `position`, stronger side first.
fn signature(position: &Position) -> Signature {
    let mut counts = [[0u8; 6]; 2];
    let mut value_of = [0u32; 2];
    for colour in Colour::ALL {
        for role in Role::ALL {
            let count = (position.by_colour(colour) & position.by_role(role)).len();
            counts[colour.index()][role.index()] = count as u8;
            value_of[colour.index()] += count * value(role);
        }
    }
    let stronger = stronger_side(&counts, &value_of);
    let text = format!(
        "{}v{}",
        side_text(&counts[stronger.index()]),
        side_text(&counts[(!stronger).index()])
    );
    Signature {
        text,
        stronger,
        counts,
        value: value_of,
    }
}

/// The side a signature writes first.
fn stronger_side(counts: &[[u8; 6]; 2], value_of: &[u32; 2]) -> Colour {
    if value_of[0] != value_of[1] {
        return if value_of[0] > value_of[1] {
            Colour::White
        } else {
            Colour::Black
        };
    }
    for role in [
        Role::Queen,
        Role::Rook,
        Role::Bishop,
        Role::Knight,
        Role::Pawn,
    ] {
        let white = counts[Colour::White.index()][role.index()];
        let black = counts[Colour::Black.index()][role.index()];
        if white != black {
            return if white > black {
                Colour::White
            } else {
                Colour::Black
            };
        }
    }
    Colour::White
}

/// One side of a signature: `KRP`.
fn side_text(counts: &[u8; 6]) -> String {
    let mut text = String::new();
    for role in SIGNATURE_ORDER {
        for _ in 0..counts[role.index()] {
            text.push(role.to_char().to_ascii_uppercase());
        }
    }
    text
}

/// The named class of a signature, [`Class::Other`] when there is no name and
/// [`Class::NotAnEnding`] when there is too much material.
fn class_of(signature: &Signature, evidence: &Evidence) -> Class {
    let strong = signature.stronger;
    let weak = !strong;
    if signature.pieces(strong) > ENDING_PIECES || signature.pieces(weak) > ENDING_PIECES {
        return Class::NotAnEnding;
    }
    let us = |role: Role| signature.count(strong, role);
    let them = |role: Role| signature.count(weak, role);
    let minors = |colour: Colour| {
        signature.count(colour, Role::Bishop) + signature.count(colour, Role::Knight)
    };
    let bare = signature.pieces(weak) == 0 && them(Role::Pawn) == 0;
    let no_pawns = signature.pawns() == 0;

    if bare {
        return match (
            us(Role::Queen),
            us(Role::Rook),
            us(Role::Bishop),
            us(Role::Knight),
            us(Role::Pawn),
        ) {
            (0, 0, 0, 0, 0) => Class::KvK,
            (1, 0, 0, 0, 0) => Class::KQvK,
            (0, 1, 0, 0, 0) => Class::KRvK,
            (0, 0, 2, 0, 0) => Class::KBBvK,
            (0, 0, 1, 1, 0) => Class::KBNvK,
            (0, 0, 0, 2, 0) => Class::KNNvK,
            (0, 0, 1, 0, 0) => Class::KBvK,
            (0, 0, 0, 1, 0) => Class::KNvK,
            (0, 0, 0, 0, 1) => Class::KPvK,
            (0, 0, 0, 0, _) => Class::Pawns,
            (0, 0, 1, 0, 1) => Class::KBPvK,
            _ => Class::Other,
        };
    }
    if signature.pieces(strong) == 0 && signature.pieces(weak) == 0 {
        return Class::Pawns;
    }
    if no_pawns {
        let named = match (
            us(Role::Queen),
            us(Role::Rook),
            us(Role::Bishop),
            us(Role::Knight),
        ) {
            (1, 0, 0, 0) => match (them(Role::Queen), them(Role::Rook), minors(weak)) {
                (1, 0, 0) => Some(Class::KQvKQ),
                (0, 1, 0) => Some(Class::KQvKR),
                (0, 0, 1) if them(Role::Bishop) == 1 => Some(Class::KQvKB),
                (0, 0, 1) => Some(Class::KQvKN),
                (0, 0, 2) => Some(Class::KQvTwoMinors),
                _ => None,
            },
            (0, 1, 0, 0) => match (them(Role::Rook), minors(weak)) {
                (1, 0) => Some(Class::KRvKR),
                (0, 1) if them(Role::Bishop) == 1 => Some(Class::KRvKB),
                (0, 1) => Some(Class::KRvKN),
                _ => None,
            },
            (0, 0, 2, 0) if them(Role::Knight) == 1 && minors(weak) == 1 => Some(Class::KBBvKN),
            (0, 0, 1, 0) if them(Role::Knight) == 1 && minors(weak) == 1 => Some(Class::KBvKN),
            (0, 0, 1, 0) if them(Role::Bishop) == 1 && minors(weak) == 1 => {
                Some(match evidence.bishops {
                    Some(bishops) if bishops.opposite_colours => Class::KBvKBOppositeColour,
                    _ => Class::KBvKBSameColour,
                })
            }
            (0, 0, 0, 1) if them(Role::Knight) == 1 && minors(weak) == 1 => Some(Class::KNvKN),
            _ => None,
        };
        if let Some(class) = named {
            return class;
        }
        if minors(strong) == 2 && them(Role::Rook) == 1 && signature.pieces(weak) == 1 {
            return Class::KRvTwoMinors;
        }
        return Class::Other;
    }
    // From here one side or the other has pawns.
    if us(Role::Pawn) == 0 && them(Role::Pawn) == 1 && signature.pieces(weak) == 0 {
        return match (
            us(Role::Queen),
            us(Role::Rook),
            us(Role::Bishop),
            us(Role::Knight),
        ) {
            (1, 0, 0, 0) => Class::KQvKP,
            (0, 1, 0, 0) => Class::KRvKP,
            (0, 0, 1, 0) => Class::KBvKP,
            (0, 0, 0, 1) => Class::KNvKP,
            _ => Class::Other,
        };
    }
    if us(Role::Rook) == 1
        && signature.pieces(strong) == 1
        && us(Role::Pawn) == 1
        && them(Role::Rook) == 1
        && signature.pieces(weak) == 1
        && them(Role::Pawn) == 0
    {
        return Class::KRPvKR;
    }
    Class::Other
}

/// The result and the method of `class`, the general case adjusted by the
/// position-specific facts that overturn it.
fn judge(class: Class, signature: &Signature, evidence: &Evidence) -> (Verdict, Technique) {
    let strong = signature.stronger;
    let weak = !strong;
    match class {
        Class::KvK
        | Class::KBvK
        | Class::KNvK
        | Class::KNNvK
        | Class::KQvKQ
        | Class::KRvKR
        | Class::KBvKN
        | Class::KBvKBSameColour
        | Class::KBvKBOppositeColour
        | Class::KNvKN => (Verdict::Draw, Technique::None),
        Class::KQvK | Class::KRvK => (Verdict::Win(strong), Technique::BoxMethod),
        Class::KBBvK => match evidence.bishops {
            Some(bishops) if bishops.same_colour => (Verdict::Draw, Technique::None),
            _ => (Verdict::Win(strong), Technique::TwoBishopMate),
        },
        Class::KBNvK => (Verdict::Win(strong), Technique::BishopKnightMate),
        Class::KQvKR | Class::KQvKB | Class::KQvKN => (Verdict::Win(strong), Technique::None),
        Class::KBBvKN => (Verdict::UsuallyWin(strong), Technique::None),
        Class::KQvKP | Class::KRvKP => (Verdict::UsuallyWin(strong), Technique::None),
        Class::KRvKB | Class::KRvKN => (Verdict::UsuallyDraw(strong), Technique::None),
        Class::KBvKP | Class::KNvKP => (Verdict::UsuallyDraw(weak), Technique::None),
        Class::KQvTwoMinors | Class::KRvTwoMinors | Class::Other | Class::NotAnEnding => {
            (Verdict::Unknown, Technique::None)
        }
        Class::Pawns => (Verdict::Unknown, Technique::Opposition),
        Class::KBPvK => match evidence.bishops {
            Some(bishops) if bishops.wrong_bishop => (Verdict::Draw, Technique::WrongBishop),
            _ => (Verdict::UsuallyWin(strong), Technique::None),
        },
        Class::KPvK => match evidence.pawn {
            Some(race) if race.rook_pawn && race.defender_holds_the_corner => {
                (Verdict::Draw, Technique::WrongRookPawn)
            }
            Some(race) if !race.defender_inside_square => {
                (Verdict::Win(strong), Technique::RuleOfTheSquare)
            }
            _ => (Verdict::UsuallyWin(strong), Technique::KeySquares),
        },
        Class::KRPvKR => match evidence.pawn {
            Some(race) if race.defender_in_front => {
                (Verdict::UsuallyDraw(strong), Technique::Philidor)
            }
            _ => (Verdict::UsuallyWin(strong), Technique::Lucena),
        },
    }
}

/// The position-specific facts of `position`.
fn evidence(position: &Position, signature: &Signature) -> Evidence {
    Evidence {
        pawn: pawn_race(position, signature),
        bishops: bishops(position, signature),
        opposition: opposition(position),
    }
}

/// The race of the only pawn, when there is exactly one.
fn pawn_race(position: &Position, signature: &Signature) -> Option<PawnRace> {
    if signature.pawns() != 1 {
        return None;
    }
    let pawn = position.by_role(Role::Pawn).first()?;
    let colour = position.piece_at(pawn)?.colour;
    let promotion = Square::new(pawn.file(), Rank::Eighth.relative_to(colour));
    let rank = pawn.rank().relative_to(colour).index() as u32;
    let steps = if rank == 1 { 5 } else { 7 - rank };
    let defender = position.king_of(!colour);
    let attacker = position.king_of(colour);
    let head_start = u32::from(position.side_to_move() != colour);
    let ahead = |king: Square| {
        king.file() == pawn.file()
            && king.rank().relative_to(colour).index() > pawn.rank().relative_to(colour).index()
    };
    Some(PawnRace {
        pawn,
        colour,
        promotion,
        rook_pawn: pawn.file() == File::A || pawn.file() == File::H,
        steps,
        defender_inside_square: distance(defender, promotion) <= steps + head_start,
        attacker_in_front: ahead(attacker),
        defender_in_front: ahead(defender),
        defender_holds_the_corner: distance(defender, promotion) <= 1,
    })
}

/// The bishops on the board, when at least one stands on it.
fn bishops(position: &Position, signature: &Signature) -> Option<Bishops> {
    let all = position.by_role(Role::Bishop);
    if all.is_empty() {
        return None;
    }
    let of = |colour: Colour| all & position.by_colour(colour);
    let same_colour = all.is_subset(SquareSet::DARK) || all.is_subset(SquareSet::LIGHT);
    let one_each = of(Colour::White).len() == 1 && of(Colour::Black).len() == 1;
    Some(Bishops {
        opposite_colours: one_each && !same_colour,
        same_colour,
        wrong_bishop: wrong_bishop(position, signature),
    })
}

/// Whether the only pawn is a rook pawn of a side whose bishops all stand on
/// the colour it does not promote on.
fn wrong_bishop(position: &Position, signature: &Signature) -> bool {
    if signature.pawns() != 1 {
        return false;
    }
    let Some(pawn) = position.by_role(Role::Pawn).first() else {
        return false;
    };
    if pawn.file() != File::A && pawn.file() != File::H {
        return false;
    }
    let Some(colour) = position.piece_at(pawn).map(|piece| piece.colour) else {
        return false;
    };
    let mine = position.by_role(Role::Bishop) & position.by_colour(colour);
    if mine.is_empty() {
        return false;
    }
    let promotion = Square::new(pawn.file(), Rank::Eighth.relative_to(colour));
    mine.into_iter()
        .all(|bishop| bishop.is_dark() != promotion.is_dark())
}

/// Whether the kings stand on one line with exactly one empty square between
/// them.
fn opposition(position: &Position) -> bool {
    let corridor = between(
        position.king_of(Colour::White),
        position.king_of(Colour::Black),
    );
    corridor.len() == 1 && (corridor & position.occupied()).is_empty()
}
