use crate::board::PieceKind;
use crate::game::{Game, IllegalMove};
use crate::moves::Move;
use crate::rules;
use crate::state::GameState;

const MATE_SCORE: i32 = 1_000_000;
const SEARCH_DEPTH: u8 = 5;

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
        if moves.is_empty() {
            return None;
        }
        let mut best: Option<Move> = None;
        let mut best_score = i32::MIN;

        for mv in moves {
            let mut next = self.game.state.clone();
            rules::apply_move_unchecked(&mut next, mv);
            let score = -search(&next, SEARCH_DEPTH.saturating_sub(1));
            if best.is_none() || score > best_score {
                best = Some(mv);
                best_score = score;
            }
        }

        best
    }
}

fn search(state: &GameState, depth: u8) -> i32 {
    let moves = rules::legal_moves(state);
    if moves.is_empty() {
        return terminal_score(state);
    }
    if depth == 0 {
        return eval_material_for_side_to_move(state);
    }

    let mut best = i32::MIN;
    for mv in moves {
        let mut next = state.clone();
        rules::apply_move_unchecked(&mut next, mv);
        let score = -search(&next, depth - 1);
        if score > best {
            best = score;
        }
    }
    best
}

fn terminal_score(state: &GameState) -> i32 {
    if rules::is_in_check(state, state.side_to_move) {
        -MATE_SCORE
    } else {
        0
    }
}

fn eval_material_for_side_to_move(state: &GameState) -> i32 {
    let us = state.side_to_move;
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
