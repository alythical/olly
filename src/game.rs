use crate::{
    board::{Board, Piece},
    PlaceError,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    board: Board,
    turn: Piece,
    history: Vec<(usize, usize)>,
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Piece::Black,
            history: Vec::new(),
        }
    }

    #[must_use]
    pub fn score(&self) -> (usize, usize) {
        let mut black = 0;
        let mut white = 0;
        for (x, y) in Self::points() {
            match self.board[(x, y)] {
                Some(Piece::Black) => black += 1,
                Some(Piece::White) => white += 1,
                None => (),
            }
        }
        (black, white)
    }

    pub fn moves(&mut self, piece: Piece) -> Vec<(usize, usize)> {
        Self::points()
            .into_iter()
            .filter(|&(x, y)| self.validate(x, y, piece).is_ok())
            .collect()
    }

    fn points() -> impl IntoIterator<Item = (usize, usize)> {
        (0..Board::width()).flat_map(|x| (0..Board::width()).map(move |y| (x, y)))
    }

    /// # Errors
    /// Returns an error if the move is invalid.
    pub fn place(&mut self, x: usize, y: usize, piece: Piece) -> Result<(), PlaceError> {
        self.validate(x, y, piece)?;
        self.board[(x, y)] = Some(piece);
        self.board.flip(x, y, piece, true);
        self.history.push((x, y));
        self.turn = !self.turn;
        Ok(())
    }

    /// # Errors
    /// Returns an error if the move is invalid.
    pub fn preview(
        &mut self,
        x: usize,
        y: usize,
        piece: Piece,
    ) -> Result<Vec<(usize, usize)>, PlaceError> {
        self.validate(x, y, piece)?;
        Ok(self.board.flip(x, y, piece, false))
    }

    fn validate(&mut self, x: usize, y: usize, piece: Piece) -> Result<(), PlaceError> {
        if x >= Board::width() || y >= Board::width() {
            Err(PlaceError::OutOfBounds(x, y))
        } else {
            match (
                self.turn == piece,
                self.board.adjacent(x, y),
                self.board[(x, y)].is_none(),
            ) {
                (false, _, _) => Err(PlaceError::Turn(piece)),
                (_, false, _) => Err(PlaceError::NotAdjacent(x, y)),
                (_, _, false) => Err(PlaceError::Occupied(x, y)),
                _ if self.board.flip(x, y, piece, false).is_empty() => {
                    Err(PlaceError::NoFlips(x, y))
                }
                _ => Ok(()),
            }
        }
    }

    pub fn over(&mut self) -> bool {
        self.moves(self.turn).is_empty()
    }

    #[must_use]
    pub fn history(&self) -> Vec<(usize, usize)> {
        self.history.clone()
    }

    #[must_use]
    pub fn turn(&self) -> Piece {
        self.turn
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Turn: {:?}", self.turn)?;
        writeln!(f, "Board:")?;
        writeln!(f, "{:?}", self.board)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, Piece, PlaceError};

    #[test]
    fn new() {
        let state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        assert_eq!(state.score(), (2, 2));
    }

    #[test]
    fn initial_moves() {
        let mut state = Game::new();
        let moves = state.moves(Piece::Black);
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn flips_preview() {
        let mut state = Game::new();
        let flips = state.preview(2, 3, Piece::Black);
        assert_eq!(flips.unwrap(), vec![(3, 3)]);
    }

    #[test]
    fn valid_placement() {
        let mut state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        let outcome = state.place(2, 3, Piece::Black);
        assert!(outcome.is_ok());
        assert_eq!(state.turn, Piece::White);
    }

    #[test]
    fn out_of_turn() {
        let mut state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        let outcome = state.place(2, 3, Piece::White);
        assert_eq!(outcome.unwrap_err(), PlaceError::Turn(Piece::White));
    }

    #[test]
    fn occupied() {
        let mut state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        let outcome = state.place(3, 3, Piece::Black);
        assert_eq!(outcome.unwrap_err(), PlaceError::Occupied(3, 3));
    }

    #[test]
    fn not_adjacent() {
        let mut state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        let outcome = state.place(0, 0, Piece::Black);
        assert_eq!(outcome.unwrap_err(), PlaceError::NotAdjacent(0, 0));
    }

    #[test]
    fn out_of_bounds() {
        let mut state = Game::new();
        assert_eq!(state.turn, Piece::Black);
        let outcome = state.place(8, 8, Piece::Black);
        assert_eq!(outcome.unwrap_err(), PlaceError::OutOfBounds(8, 8));
    }
}
