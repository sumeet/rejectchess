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
    pub white_king: Square,
    pub black_king: Square,
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
            white_king: (4, 0),
            black_king: (4, 7),
        }
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut parts = fen.split_whitespace();

        let board_str = parts.next()?;
        let mut board: Board = [[None; 8]; 8];
        let mut white_king = (0, 0);
        let mut black_king = (0, 0);

        for (rank_idx, rank_str) in board_str.split('/').enumerate() {
            let rank = 7 - rank_idx;
            let mut file = 0usize;
            for c in rank_str.chars() {
                if let Some(skip) = c.to_digit(10) {
                    file += skip as usize;
                } else {
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let kind = match c.to_ascii_lowercase() {
                        'p' => PieceKind::Pawn,
                        'n' => PieceKind::Knight,
                        'b' => PieceKind::Bishop,
                        'r' => PieceKind::Rook,
                        'q' => PieceKind::Queen,
                        'k' => {
                            if color == Color::White {
                                white_king = (file as u8, rank as u8);
                            } else {
                                black_king = (file as u8, rank as u8);
                            }
                            PieceKind::King
                        }
                        _ => return None,
                    };
                    board[rank][file] = Some(Piece { color, kind });
                    file += 1;
                }
            }
        }

        let side_str = parts.next()?;
        let side_to_move = match side_str {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };

        let castling_str = parts.next().unwrap_or("-");
        let castling = CastlingRights {
            white_kingside: castling_str.contains('K'),
            white_queenside: castling_str.contains('Q'),
            black_kingside: castling_str.contains('k'),
            black_queenside: castling_str.contains('q'),
        };

        let ep_str = parts.next().unwrap_or("-");
        let en_passant = if ep_str == "-" {
            None
        } else {
            let bytes = ep_str.as_bytes();
            if bytes.len() == 2 {
                let file = bytes[0].wrapping_sub(b'a');
                let rank = bytes[1].wrapping_sub(b'1');
                if file < 8 && rank < 8 {
                    Some((file, rank))
                } else {
                    None
                }
            } else {
                None
            }
        };

        Some(Self {
            board,
            side_to_move,
            castling,
            en_passant,
            white_king,
            black_king,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let state = GameState::from_fen(fen).unwrap();

        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.white_king, (4, 0));
        assert_eq!(state.black_king, (4, 7));
        assert!(state.castling.white_kingside);
        assert!(state.castling.white_queenside);
        assert!(state.castling.black_kingside);
        assert!(state.castling.black_queenside);
        assert!(state.en_passant.is_none());
    }

    #[test]
    fn from_fen_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let state = GameState::from_fen(fen).unwrap();

        assert_eq!(state.side_to_move, Color::Black);
        assert_eq!(state.en_passant, Some((4, 2)));
    }

    #[test]
    fn from_fen_partial_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 0 1";
        let state = GameState::from_fen(fen).unwrap();

        assert!(state.castling.white_kingside);
        assert!(!state.castling.white_queenside);
        assert!(!state.castling.black_kingside);
        assert!(state.castling.black_queenside);
    }

    #[test]
    fn from_fen_invalid_returns_none() {
        assert!(GameState::from_fen("invalid").is_none());
        assert!(GameState::from_fen("").is_none());
    }
}
