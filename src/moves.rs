use crate::board::{PieceKind, Square};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MoveKind {
    Quiet,
    Capture,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    Promotion,
    PromotionCapture,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
    pub kind: MoveKind,
}
