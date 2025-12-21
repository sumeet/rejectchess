#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

pub type Square = (u8, u8); // file, rank
pub type Board = [[Option<Piece>; 8]; 8];

pub fn in_bounds(file: i8, rank: i8) -> bool {
    file >= 0 && file < 8 && rank >= 0 && rank < 8
}

pub fn piece_at(board: &Board, sq: Square) -> Option<Piece> {
    board[sq.1 as usize][sq.0 as usize]
}

pub fn set_piece(board: &mut Board, sq: Square, piece: Option<Piece>) {
    board[sq.1 as usize][sq.0 as usize] = piece;
}
