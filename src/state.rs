use crate::board::{Board, Color, Piece, PieceKind, Square};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub board: Board,
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
}

impl GameState {
    pub fn new() -> Self {
        let mut board = [[None; 8]; 8];
        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];

        for (file, kind) in back_rank.iter().enumerate() {
            board[0][file] = Some(Piece {
                color: Color::White,
                kind: *kind,
            });
            board[7][file] = Some(Piece {
                color: Color::Black,
                kind: *kind,
            });
        }

        for file in 0..8 {
            board[1][file] = Some(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            });
            board[6][file] = Some(Piece {
                color: Color::Black,
                kind: PieceKind::Pawn,
            });
        }

        Self {
            board,
            side_to_move: Color::White,
            castling: CastlingRights::new(),
            en_passant: None,
        }
    }
}
