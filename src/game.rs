use crate::moves::Move;
use crate::rules;
use crate::state::GameState;

#[derive(Debug)]
pub struct IllegalMove;

pub struct Game {
    pub state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        Some(Self {
            state: GameState::from_fen(fen)?,
        })
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        rules::legal_moves(&self.state)
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), IllegalMove> {
        if rules::is_move_legal(&self.state, mv) {
            rules::apply_move_unchecked(&mut self.state, mv);
            Ok(())
        } else {
            Err(IllegalMove)
        }
    }

    pub fn is_checkmate(&self) -> bool {
        rules::is_checkmate(&self.state)
    }

    pub fn is_stalemate(&self) -> bool {
        rules::is_stalemate(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::Game;
    use crate::rules;

    #[test]
    fn initial_position_has_20_legal_moves() {
        let game = Game::new();
        assert_eq!(game.legal_moves().len(), 20);
    }

    #[test]
    fn initial_position_not_in_check() {
        let game = Game::new();
        assert!(!rules::is_in_check(&game.state, game.state.side_to_move));
    }
}
