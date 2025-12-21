use rand::seq::SliceRandom;

use crate::game::{Game, IllegalMove};
use crate::moves::Move;

pub struct Engine {
    game: Game,
}

impl Engine {
    pub fn new() -> Self {
        Self { game: Game::new() }
    }

    pub fn reset(&mut self) {
        self.game = Game::new();
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        self.game.legal_moves()
    }

    pub fn apply_moves(&mut self, moves: &[Move]) -> Result<(), IllegalMove> {
        for mv in moves {
            self.game.make_move(*mv)?;
        }
        Ok(())
    }

    pub fn go(&self) -> Option<Move> {
        let moves = self.game.legal_moves();
        let mut rng = rand::thread_rng();
        moves.choose(&mut rng).copied()
    }
}
