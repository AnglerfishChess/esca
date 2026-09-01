//! A variant, a start position, and the moves played from it.

use core::fmt;
use std::sync::Arc;

use crate::error::{FenError, IllegalMove, MoveParseError, PositionError};
use crate::moves::{Move, MoveList};
use crate::position::Position;
use crate::variant::{CastlingOutput, DrawClaim, Outcome, Variant};

/// A played game: the positions reached, and therefore the only thing that
/// can answer repetition and claim questions.
#[derive(Clone)]
pub struct Game {
    variant: Arc<dyn Variant>,
    positions: Vec<Position>,
    keys: Vec<u64>,
    moves: Vec<Move>,
    castling_output: CastlingOutput,
    claims: Vec<DrawClaim>,
}

impl Game {
    /// A game from `variant.start_position(0)`.
    pub fn new(variant: Arc<dyn Variant>) -> Game {
        Game::with_seed(variant, 0)
    }

    /// A game from `variant.start_position(seed)`.
    pub fn with_seed(variant: Arc<dyn Variant>, seed: u64) -> Game {
        let start = variant.start_position(seed);
        Game::seeded(variant, start)
    }

    /// A game starting from `start`.
    pub fn from_position(
        variant: Arc<dyn Variant>,
        start: Position,
    ) -> Result<Game, PositionError> {
        variant.validate(&start)?;
        Ok(Game::seeded(variant, start))
    }

    /// A game starting from the position `fen` describes.
    pub fn from_fen(variant: Arc<dyn Variant>, fen: &str) -> Result<Game, FenError> {
        let start = Position::from_fen(fen)?;
        Game::from_position(variant, start).map_err(|_| FenError::Position)
    }

    fn seeded(variant: Arc<dyn Variant>, start: Position) -> Game {
        let key = start.repetition_key();
        let mut game = Game {
            variant,
            positions: vec![start],
            keys: vec![key],
            moves: Vec::new(),
            castling_output: CastlingOutput::default(),
            claims: Vec::new(),
        };
        game.refresh_claims();
        game
    }

    /// The rules this game is played under.
    pub fn variant(&self) -> &dyn Variant {
        self.variant.as_ref()
    }

    /// The castling spelling of this game's UCI output.
    pub fn castling_output(&self) -> CastlingOutput {
        self.castling_output
    }

    /// Sets the castling spelling of this game's UCI output.
    pub fn set_castling_output(&mut self, style: CastlingOutput) {
        self.castling_output = style;
    }

    /// The UCI text of `mv` in the current position.
    pub fn move_to_uci(&self, mv: Move) -> String {
        self.variant
            .move_to_uci(self.position(), mv, self.castling_output)
    }

    /// The SAN text of `mv` in the current position.
    pub fn move_to_san(&self, mv: Move) -> String {
        self.variant.move_to_san(self.position(), mv)
    }

    /// The position now.
    pub fn position(&self) -> &Position {
        self.positions.last().expect("a game holds its start")
    }

    /// The position the game started from.
    pub fn start_position(&self) -> &Position {
        self.positions.first().expect("a game holds its start")
    }

    /// The moves played, in order.
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Every position from the start to the current one.
    pub fn positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.iter()
    }

    /// How many moves have been played.
    pub fn ply(&self) -> u32 {
        self.moves.len() as u32
    }

    /// The legal moves in the current position.
    pub fn legal_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.variant.legal_moves(self.position(), &mut moves);
        moves
    }

    /// Plays `mv`.
    pub fn play(&mut self, mv: Move) -> Result<(), IllegalMove> {
        if !self.variant.is_legal(self.position(), mv) {
            return Err(IllegalMove);
        }
        let next = self.variant.play(self.position(), mv);
        self.keys.push(next.repetition_key());
        self.positions.push(next);
        self.moves.push(mv);
        self.refresh_claims();
        Ok(())
    }

    /// Plays the move `text` names in UCI notation.
    pub fn play_uci(&mut self, text: &str) -> Result<(), MoveParseError> {
        let mv = self.variant.move_from_uci(self.position(), text)?;
        self.play(mv).expect("a parsed move is legal");
        Ok(())
    }

    /// Plays the move `text` names in SAN.
    pub fn play_san(&mut self, text: &str) -> Result<(), MoveParseError> {
        let mv = self.variant.move_from_san(self.position(), text)?;
        self.play(mv).expect("a parsed move is legal");
        Ok(())
    }

    /// Takes back the last move, returning it.
    pub fn undo(&mut self) -> Option<Move> {
        let mv = self.moves.pop()?;
        self.positions.pop();
        self.keys.pop();
        self.refresh_claims();
        Some(mv)
    }

    /// The automatic terminal conditions, repetition included.
    pub fn outcome(&self) -> Option<Outcome> {
        let outcome = self.variant.outcome(self.position());
        match outcome {
            Some(Outcome::Checkmate { .. }) | Some(Outcome::Stalemate) => outcome,
            _ if self.repetitions() >= 5 => Some(Outcome::FivefoldRepetition),
            _ => outcome,
        }
    }

    /// The draws a player could claim now, in no particular order.
    pub fn claims(&self) -> &[DrawClaim] {
        &self.claims
    }

    /// How often the current position has occurred in this game.
    pub fn repetitions(&self) -> u32 {
        let key = *self.keys.last().expect("a game holds its start");
        self.keys.iter().filter(|&&other| other == key).count() as u32
    }

    fn refresh_claims(&mut self) {
        self.claims.clear();
        if self.position().halfmove_clock() >= 100 {
            self.claims.push(DrawClaim::FiftyMoves);
        }
        if self.repetitions() >= 3 {
            self.claims.push(DrawClaim::ThreefoldRepetition);
        }
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Game")
            .field("variant", &self.variant.name())
            .field("start", &self.start_position().fen())
            .field("moves", &self.moves)
            .finish()
    }
}
