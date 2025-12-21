use crate::board::{in_bounds, piece_at, Color, Piece, PieceKind, Square};
use crate::dirs::{BISHOP_DIRS, KING_DIRS, KNIGHT_DIRS, QUEEN_DIRS, ROOK_DIRS};
use crate::moves::{Move, MoveKind};
use crate::state::GameState;

const PROMOTION_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];

pub fn generate_candidates(state: &GameState) -> Vec<Move> {
    let mut moves = Vec::new();
    for rank in 0..8 {
        for file in 0..8 {
            let from = (file as u8, rank as u8);
            if let Some(piece) = piece_at(&state.board, from) {
                if piece.color != state.side_to_move {
                    continue;
                }
                match piece.kind {
                    PieceKind::Pawn => gen_pawn_moves(state, from, &mut moves),
                    PieceKind::Knight => gen_knight_moves(state, from, &mut moves),
                    PieceKind::Bishop => gen_slider_moves(state, from, &mut moves, &BISHOP_DIRS),
                    PieceKind::Rook => gen_slider_moves(state, from, &mut moves, &ROOK_DIRS),
                    PieceKind::Queen => gen_slider_moves(state, from, &mut moves, &QUEEN_DIRS),
                    PieceKind::King => gen_king_moves(state, from, &mut moves),
                }
            }
        }
    }
    moves
}

fn gen_pawn_moves(state: &GameState, from: Square, moves: &mut Vec<Move>) {
    let piece = piece_at(&state.board, from).expect("missing pawn");
    let dir: i8 = if piece.color == Color::White { 1 } else { -1 };
    let start_rank: u8 = if piece.color == Color::White { 1 } else { 6 };
    let last_rank: u8 = if piece.color == Color::White { 7 } else { 0 };

    let file = from.0 as i8;
    let rank = from.1 as i8;

    let one_rank = rank + dir;
    if in_bounds(file, one_rank) {
        let to = (file as u8, one_rank as u8);
        if piece_at(&state.board, to).is_none() {
            if to.1 == last_rank {
                add_promotion_moves(moves, from, to);
            } else {
                push_move(moves, from, to, MoveKind::Normal);
                if from.1 == start_rank {
                    let two_rank = rank + dir * 2;
                    let to_two = (file as u8, two_rank as u8);
                    if in_bounds(file, two_rank) && piece_at(&state.board, to_two).is_none() {
                        push_move(moves, from, to_two, MoveKind::Normal);
                    }
                }
            }
        }
    }

    for df in [-1, 1] {
        let nf = file + df;
        let nr = rank + dir;
        if !in_bounds(nf, nr) {
            continue;
        }
        let to = (nf as u8, nr as u8);
        if let Some(target) = piece_at(&state.board, to) {
            if target.color != piece.color {
                if to.1 == last_rank {
                    add_promotion_moves(moves, from, to);
                } else {
                    push_move(moves, from, to, MoveKind::Normal);
                }
            }
        }
    }

    if let Some(ep) = state.en_passant {
        let ep_file = ep.0 as i8;
        let ep_rank = ep.1 as i8;
        if ep_rank == rank + dir && (ep_file - file).abs() == 1 {
            push_move(moves, from, ep, MoveKind::EnPassant);
        }
    }
}

fn gen_knight_moves(state: &GameState, from: Square, moves: &mut Vec<Move>) {
    let piece = piece_at(&state.board, from).expect("missing knight");
    let file = from.0 as i8;
    let rank = from.1 as i8;

    for (df, dr) in KNIGHT_DIRS {
        let nf = file + df;
        let nr = rank + dr;
        if !in_bounds(nf, nr) {
            continue;
        }
        let to = (nf as u8, nr as u8);
        add_step_move(state, piece, from, to, moves);
    }
}

fn gen_slider_moves(
    state: &GameState,
    from: Square,
    moves: &mut Vec<Move>,
    dirs: &[(i8, i8)],
) {
    let piece = piece_at(&state.board, from).expect("missing slider");
    let file = from.0 as i8;
    let rank = from.1 as i8;

    for (df, dr) in dirs {
        let mut nf = file + df;
        let mut nr = rank + dr;
        while in_bounds(nf, nr) {
            let to = (nf as u8, nr as u8);
            if let Some(target) = piece_at(&state.board, to) {
                if target.color != piece.color {
                    push_move(moves, from, to, MoveKind::Normal);
                }
                break;
            } else {
                push_move(moves, from, to, MoveKind::Normal);
            }
            nf += df;
            nr += dr;
        }
    }
}

fn gen_king_moves(state: &GameState, from: Square, moves: &mut Vec<Move>) {
    let piece = piece_at(&state.board, from).expect("missing king");
    let file = from.0 as i8;
    let rank = from.1 as i8;

    for (df, dr) in KING_DIRS {
        let nf = file + df;
        let nr = rank + dr;
        if !in_bounds(nf, nr) {
            continue;
        }
        let to = (nf as u8, nr as u8);
        add_step_move(state, piece, from, to, moves);
    }

    let rook = Piece {
        color: piece.color,
        kind: PieceKind::Rook,
    };
    match piece.color {
        Color::White => {
            if from == (4, 0) && state.castling.white_kingside {
                if piece_at(&state.board, (5, 0)).is_none()
                    && piece_at(&state.board, (6, 0)).is_none()
                    && piece_at(&state.board, (7, 0)) == Some(rook)
                {
                    push_move(moves, from, (6, 0), MoveKind::CastleKingside);
                }
            }
            if from == (4, 0) && state.castling.white_queenside {
                if piece_at(&state.board, (1, 0)).is_none()
                    && piece_at(&state.board, (2, 0)).is_none()
                    && piece_at(&state.board, (3, 0)).is_none()
                    && piece_at(&state.board, (0, 0)) == Some(rook)
                {
                    push_move(moves, from, (2, 0), MoveKind::CastleQueenside);
                }
            }
        }
        Color::Black => {
            if from == (4, 7) && state.castling.black_kingside {
                if piece_at(&state.board, (5, 7)).is_none()
                    && piece_at(&state.board, (6, 7)).is_none()
                    && piece_at(&state.board, (7, 7)) == Some(rook)
                {
                    push_move(moves, from, (6, 7), MoveKind::CastleKingside);
                }
            }
            if from == (4, 7) && state.castling.black_queenside {
                if piece_at(&state.board, (1, 7)).is_none()
                    && piece_at(&state.board, (2, 7)).is_none()
                    && piece_at(&state.board, (3, 7)).is_none()
                    && piece_at(&state.board, (0, 7)) == Some(rook)
                {
                    push_move(moves, from, (2, 7), MoveKind::CastleQueenside);
                }
            }
        }
    }
}

fn add_step_move(state: &GameState, piece: Piece, from: Square, to: Square, moves: &mut Vec<Move>) {
    if let Some(target) = piece_at(&state.board, to) {
        if target.color != piece.color {
            push_move(moves, from, to, MoveKind::Normal);
        }
    } else {
        push_move(moves, from, to, MoveKind::Normal);
    }
}

fn add_promotion_moves(moves: &mut Vec<Move>, from: Square, to: Square) {
    for promo in PROMOTION_PIECES {
        push_move(moves, from, to, MoveKind::Promotion(promo));
    }
}

fn push_move(moves: &mut Vec<Move>, from: Square, to: Square, kind: MoveKind) {
    moves.push(Move { from, to, kind });
}
