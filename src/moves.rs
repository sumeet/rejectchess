use crate::board::{PieceKind, Square};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MoveKind {
    Normal,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    Promotion(PieceKind),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub kind: MoveKind,
}
