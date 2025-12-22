use rayon::prelude::*;

use crate::board::{PieceKind, piece_at};
use crate::game::{Game, IllegalMove};
use crate::movegen;
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
        let moves = ordered_candidates(&self.game.state);
        if moves.is_empty() {
            return None;
        }

        let depth = SEARCH_DEPTH.saturating_sub(1);
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut first_index = None;

        for (idx, mv) in moves.iter().copied().enumerate() {
            if let Some(next) = rules::try_apply_legal(&self.game.state, mv) {
                best_score = -search_ab(&next, depth, -INF, INF);
                best_move = Some(mv);
                first_index = Some(idx);
                break;
            }
        }

        let Some(start) = first_index else {
            return None;
        };

        if start + 1 < moves.len() {
            let alpha0 = best_score;
            if let Some((score, mv)) = moves[start + 1..]
                .par_iter()
                .filter_map(|&mv| {
                    let next = rules::try_apply_legal(&self.game.state, mv)?;
                    let score = -search_ab(&next, depth, -INF, -alpha0);
                    Some((score, mv))
                })
                .max_by_key(|(score, _)| *score)
            {
                if score > best_score {
                    best_move = Some(mv);
                }
            }
        }

        best_move
    }
}

fn search_ab(state: &GameState, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    let moves = ordered_candidates(state);
    if depth == 0 {
        return eval_material_for_side_to_move(state);
    }

    let mut best = i32::MIN;
    let mut found_legal = false;
    for mv in moves {
        let Some(next) = rules::try_apply_legal(state, mv) else {
            continue;
        };
        found_legal = true;
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
    if !found_legal {
        return terminal_score(state);
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

fn move_order_key(state: &GameState, mv: Move) -> u8 {
    let is_promo = matches!(mv.kind, MoveKind::Promotion(_));
    let is_capture = match mv.kind {
        MoveKind::EnPassant => true,
        MoveKind::CastleKingside | MoveKind::CastleQueenside => false,
        MoveKind::Promotion(_) | MoveKind::Normal => piece_at(&state.board, mv.to).is_some(),
    };
    let mut key = 3;
    if is_promo {
        key -= 2;
    }
    if is_capture {
        key -= 1;
    }
    key
}

fn ordered_candidates(state: &GameState) -> Vec<Move> {
    let mut moves = movegen::generate_candidates(state);
    moves.sort_by_key(|mv| move_order_key(state, *mv));
    moves
}
