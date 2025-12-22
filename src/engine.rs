use rayon::prelude::*;

use crate::board::{PieceKind, piece_at};
use crate::game::{Game, IllegalMove};
use crate::moves::{Move, MoveKind};
use crate::rules;
use crate::state::GameState;

const MATE_SCORE: i32 = 1_000_000;
const INF: i32 = 1_000_000_000;
const SEARCH_DEPTH: u8 = 7;

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
        let mut moves = self.game.legal_moves();
        if moves.is_empty() {
            return None;
        }

        moves.sort_by_key(|mv| move_order_key(&self.game.state, *mv));

        let depth = SEARCH_DEPTH.saturating_sub(1);
        let first = moves[0];
        let mut next = self.game.state.clone();
        rules::apply_move_unchecked(&mut next, first);
        let best_score = -search_ab(&next, depth, -INF, INF);
        let mut best_move = first;

        if moves.len() > 1 {
            let alpha0 = best_score;
            if let Some((score, mv)) = moves[1..]
                .par_iter()
                .map(|&mv| {
                    let mut next = self.game.state.clone();
                    rules::apply_move_unchecked(&mut next, mv);
                    let score = -search_ab(&next, depth, -INF, -alpha0);
                    (score, mv)
                })
                .max_by_key(|(score, _)| *score)
            {
                if score > best_score {
                    best_move = mv;
                }
            }
        }

        Some(best_move)
    }
}

fn search_ab(state: &GameState, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    let moves = rules::legal_move_states(state);
    if moves.is_empty() {
        return terminal_score(state);
    }
    if depth == 0 {
        return eval_material_for_side_to_move(state);
    }

    let mut best = i32::MIN;
    for (_mv, next) in moves {
        let score = -search_ab(&next, depth - 1, -beta, -alpha);
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
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

fn move_order_key(state: &GameState, mv: Move) -> (u8, u8, u8) {
    let is_promo = matches!(mv.kind, MoveKind::Promotion(_));
    let is_capture = match mv.kind {
        MoveKind::EnPassant => true,
        MoveKind::CastleKingside | MoveKind::CastleQueenside => false,
        MoveKind::Promotion(_) | MoveKind::Normal => piece_at(&state.board, mv.to).is_some(),
    };
    let gives_check = {
        let mut next = state.clone();
        rules::apply_move_unchecked(&mut next, mv);
        rules::is_in_check(&next, next.side_to_move)
    };

    (
        if is_promo { 0 } else { 1 },
        if is_capture { 0 } else { 1 },
        if gives_check { 0 } else { 1 },
    )
}
