use crate::board::{Color, PieceKind};
use crate::game::{Game, IllegalMove};
use crate::moves::Move;
use crate::rules;
use crate::state::GameState;

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
        let us = self.game.state.side_to_move;
        let moves = self.game.legal_moves();
        let mut best: Option<Move> = None;
        let mut best_score = i32::MIN;

        for mv in moves {
            let mut next = self.game.state.clone();
            rules::apply_move_unchecked(&mut next, mv);
            if rules::is_checkmate(&next) {
                return Some(mv);
            }
            let score = eval_material(&next, us);
            if best.is_none() || score > best_score {
                best = Some(mv);
                best_score = score;
            }
        }

        best
    }
}

fn eval_material(state: &GameState, us: Color) -> i32 {
    let mut score = 0;
    for rank in 0..8 {
        for file in 0..8 {
            if let Some(piece) = state.board[rank][file] {
                let value = match piece.kind {
                    PieceKind::Pawn => 1,
                    PieceKind::Knight => 3,
                    PieceKind::Bishop => 3,
                    PieceKind::Rook => 5,
                    PieceKind::Queen => 9,
                    PieceKind::King => 0,
                };
                if piece.color == us {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
    }
    score
}
