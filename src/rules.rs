use crate::board::{in_bounds, piece_at, set_piece, Board, Color, Piece, PieceKind, Square};
use crate::dirs::{BISHOP_DIRS, KING_DIRS, KNIGHT_DIRS, ROOK_DIRS};
use crate::movegen;
use crate::moves::{Move, MoveKind};
use crate::state::{CastlingRights, GameState};

pub fn legal_moves(state: &GameState) -> Vec<Move> {
    movegen::generate_candidates(state)
        .into_iter()
        .filter(|mv| is_move_legal(state, *mv))
        .collect()
}

pub fn is_checkmate(state: &GameState) -> bool {
    is_in_check(state, state.side_to_move) && legal_moves(state).is_empty()
}

pub fn is_stalemate(state: &GameState) -> bool {
    !is_in_check(state, state.side_to_move) && legal_moves(state).is_empty()
}

pub fn is_move_legal(state: &GameState, mv: Move) -> bool {
    let mover = state.side_to_move;
    if matches!(mv.kind, MoveKind::CastleKingside | MoveKind::CastleQueenside) {
        if is_in_check(state, mover) {
            return false;
        }
        if king_passes_through_check(state, mv) {
            return false;
        }
    }

    let mut next = state.clone();
    apply_move_unchecked(&mut next, mv);
    !is_in_check(&next, mover)
}

pub fn apply_move_unchecked(state: &mut GameState, mv: Move) {
    let from = mv.from;
    let to = mv.to;
    let moving_piece = piece_at(&state.board, from).expect("missing piece");

    state.en_passant = None;

    let mut captured_square: Option<Square> = None;
    let mut captured_piece: Option<Piece> = None;

    match mv.kind {
        MoveKind::CastleKingside | MoveKind::CastleQueenside => {
            let (rank, rook_from_file, rook_to_file, king_to_file) = match moving_piece.color {
                Color::White => {
                    if mv.kind == MoveKind::CastleKingside {
                        (0, 7, 5, 6)
                    } else {
                        (0, 0, 3, 2)
                    }
                }
                Color::Black => {
                    if mv.kind == MoveKind::CastleKingside {
                        (7, 7, 5, 6)
                    } else {
                        (7, 0, 3, 2)
                    }
                }
            };
            let king_to = (king_to_file, rank);
            let rook_from = (rook_from_file, rank);
            let rook_to = (rook_to_file, rank);
            let rook = piece_at(&state.board, rook_from).expect("missing rook");

            set_piece(&mut state.board, from, None);
            set_piece(&mut state.board, king_to, Some(moving_piece));
            set_piece(&mut state.board, rook_from, None);
            set_piece(&mut state.board, rook_to, Some(rook));
        }
        MoveKind::EnPassant => {
            let capture_sq = (to.0, from.1);
            captured_square = Some(capture_sq);
            captured_piece = piece_at(&state.board, capture_sq);
            set_piece(&mut state.board, capture_sq, None);
            set_piece(&mut state.board, from, None);
            set_piece(&mut state.board, to, Some(moving_piece));
        }
        MoveKind::Promotion(promo) => {
            if let Some(target) = piece_at(&state.board, to) {
                captured_square = Some(to);
                captured_piece = Some(target);
            }
            let promoted = Piece {
                color: moving_piece.color,
                kind: promo,
            };
            set_piece(&mut state.board, from, None);
            set_piece(&mut state.board, to, Some(promoted));
        }
        MoveKind::Normal => {
            if let Some(target) = piece_at(&state.board, to) {
                captured_square = Some(to);
                captured_piece = Some(target);
            }
            set_piece(&mut state.board, from, None);
            set_piece(&mut state.board, to, Some(moving_piece));
        }
    }

    update_castling_rights_on_move(state, moving_piece, from);
    if let (Some(square), Some(piece)) = (captured_square, captured_piece) {
        update_castling_rights_on_capture(state, square, piece);
    }

    if moving_piece.kind == PieceKind::Pawn && mv.kind == MoveKind::Normal {
        let rank_diff = (to.1 as i8 - from.1 as i8).abs();
        if from.0 == to.0 && rank_diff == 2 {
            let mid_rank = (to.1 + from.1) / 2;
            state.en_passant = Some((from.0, mid_rank));
        }
    }

    state.side_to_move = state.side_to_move.opposite();
}

pub fn is_in_check(state: &GameState, color: Color) -> bool {
    let king_sq = find_king(&state.board, color).expect("missing king");
    is_square_attacked(state, king_sq, color.opposite())
}

pub fn is_square_attacked(state: &GameState, square: Square, by_color: Color) -> bool {
    let file = square.0 as i8;
    let rank = square.1 as i8;

    let pawn_dir: i8 = if by_color == Color::White { -1 } else { 1 };
    for df in [-1, 1] {
        let nf = file + df;
        let nr = rank + pawn_dir;
        if in_bounds(nf, nr) {
            let sq = (nf as u8, nr as u8);
            if piece_at(&state.board, sq)
                == Some(Piece {
                    color: by_color,
                    kind: PieceKind::Pawn,
                })
            {
                return true;
            }
        }
    }

    for (df, dr) in KNIGHT_DIRS {
        let nf = file + df;
        let nr = rank + dr;
        if in_bounds(nf, nr) {
            let sq = (nf as u8, nr as u8);
            if piece_at(&state.board, sq)
                == Some(Piece {
                    color: by_color,
                    kind: PieceKind::Knight,
                })
            {
                return true;
            }
        }
    }

    for (df, dr) in KING_DIRS {
        let nf = file + df;
        let nr = rank + dr;
        if in_bounds(nf, nr) {
            let sq = (nf as u8, nr as u8);
            if piece_at(&state.board, sq)
                == Some(Piece {
                    color: by_color,
                    kind: PieceKind::King,
                })
            {
                return true;
            }
        }
    }

    diagonal_attacked(state, square, by_color) || orthogonal_attacked(state, square, by_color)
}

fn diagonal_attacked(state: &GameState, square: Square, by_color: Color) -> bool {
    let file = square.0 as i8;
    let rank = square.1 as i8;
    for (df, dr) in BISHOP_DIRS {
        let mut nf = file + df;
        let mut nr = rank + dr;
        while in_bounds(nf, nr) {
            let sq = (nf as u8, nr as u8);
            if let Some(piece) = piece_at(&state.board, sq) {
                if piece.color == by_color
                    && matches!(piece.kind, PieceKind::Bishop | PieceKind::Queen)
                {
                    return true;
                }
                break;
            }
            nf += df;
            nr += dr;
        }
    }
    false
}

fn orthogonal_attacked(state: &GameState, square: Square, by_color: Color) -> bool {
    let file = square.0 as i8;
    let rank = square.1 as i8;
    for (df, dr) in ROOK_DIRS {
        let mut nf = file + df;
        let mut nr = rank + dr;
        while in_bounds(nf, nr) {
            let sq = (nf as u8, nr as u8);
            if let Some(piece) = piece_at(&state.board, sq) {
                if piece.color == by_color && matches!(piece.kind, PieceKind::Rook | PieceKind::Queen)
                {
                    return true;
                }
                break;
            }
            nf += df;
            nr += dr;
        }
    }
    false
}

fn find_king(board: &Board, color: Color) -> Option<Square> {
    for rank in 0..8 {
        for file in 0..8 {
            if board[rank][file]
                == Some(Piece {
                    color,
                    kind: PieceKind::King,
                })
            {
                return Some((file as u8, rank as u8));
            }
        }
    }
    None
}

fn king_passes_through_check(state: &GameState, mv: Move) -> bool {
    let mover = state.side_to_move;
    let rank = if mover == Color::White { 0 } else { 7 };
    let (mid_file, end_file) = match mv.kind {
        MoveKind::CastleKingside => (5, 6),
        MoveKind::CastleQueenside => (3, 2),
        _ => return false,
    };
    let mid_sq = (mid_file, rank);
    let end_sq = (end_file, rank);
    let opponent = mover.opposite();
    is_square_attacked(state, mid_sq, opponent) || is_square_attacked(state, end_sq, opponent)
}

fn update_castling_rights_on_move(state: &mut GameState, piece: Piece, from: Square) {
    match piece.kind {
        PieceKind::King => clear_castling_rights(&mut state.castling, piece.color),
        PieceKind::Rook => match (piece.color, from) {
            (Color::White, (0, 0)) => state.castling.white_queenside = false,
            (Color::White, (7, 0)) => state.castling.white_kingside = false,
            (Color::Black, (0, 7)) => state.castling.black_queenside = false,
            (Color::Black, (7, 7)) => state.castling.black_kingside = false,
            _ => {}
        },
        _ => {}
    }
}

fn update_castling_rights_on_capture(state: &mut GameState, square: Square, piece: Piece) {
    if piece.kind != PieceKind::Rook {
        return;
    }
    match (piece.color, square) {
        (Color::White, (0, 0)) => state.castling.white_queenside = false,
        (Color::White, (7, 0)) => state.castling.white_kingside = false,
        (Color::Black, (0, 7)) => state.castling.black_queenside = false,
        (Color::Black, (7, 7)) => state.castling.black_kingside = false,
        _ => {}
    }
}

fn clear_castling_rights(castling: &mut CastlingRights, color: Color) {
    match color {
        Color::White => {
            castling.white_kingside = false;
            castling.white_queenside = false;
        }
        Color::Black => {
            castling.black_kingside = false;
            castling.black_queenside = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::set_piece;

    fn empty_state(side: Color) -> GameState {
        GameState {
            board: [[None; 8]; 8],
            side_to_move: side,
            castling: CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: false,
                black_queenside: false,
            },
            en_passant: None,
        }
    }

    #[test]
    fn castling_through_check_is_illegal() {
        let mut state = empty_state(Color::White);
        state.castling.white_kingside = true;

        set_piece(
            &mut state.board,
            (4, 0),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::King,
            }),
        );
        set_piece(
            &mut state.board,
            (7, 0),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Rook,
            }),
        );
        set_piece(
            &mut state.board,
            (5, 7),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::Rook,
            }),
        );
        set_piece(
            &mut state.board,
            (4, 7),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::King,
            }),
        );

        let moves = legal_moves(&state);
        assert!(!moves
            .iter()
            .any(|mv| matches!(mv.kind, MoveKind::CastleKingside)));
    }

    #[test]
    fn en_passant_exposing_check_is_illegal() {
        let mut state = empty_state(Color::White);
        state.en_passant = Some((3, 5));

        set_piece(
            &mut state.board,
            (4, 0),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::King,
            }),
        );
        set_piece(
            &mut state.board,
            (4, 4),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            }),
        );
        set_piece(
            &mut state.board,
            (3, 4),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::Pawn,
            }),
        );
        set_piece(
            &mut state.board,
            (4, 7),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::Rook,
            }),
        );

        let moves = legal_moves(&state);
        assert!(!moves.iter().any(|mv| {
            mv.from == (4, 4)
                && mv.to == (3, 5)
                && matches!(mv.kind, MoveKind::EnPassant)
        }));
    }

    #[test]
    fn promotion_moves_generated() {
        let mut state = empty_state(Color::White);

        set_piece(
            &mut state.board,
            (4, 0),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::King,
            }),
        );
        set_piece(
            &mut state.board,
            (4, 7),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::King,
            }),
        );
        set_piece(
            &mut state.board,
            (0, 6),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            }),
        );
        set_piece(
            &mut state.board,
            (1, 7),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::Rook,
            }),
        );

        let moves = legal_moves(&state);
        let quiet_promotions = moves
            .iter()
            .filter(|mv| {
                mv.from == (0, 6)
                    && mv.to == (0, 7)
                    && matches!(mv.kind, MoveKind::Promotion(_))
            })
            .count();
        let capture_promotions = moves
            .iter()
            .filter(|mv| {
                mv.from == (0, 6)
                    && mv.to == (1, 7)
                    && matches!(mv.kind, MoveKind::Promotion(_))
            })
            .count();

        assert_eq!(quiet_promotions, 4);
        assert_eq!(capture_promotions, 4);
    }
}
