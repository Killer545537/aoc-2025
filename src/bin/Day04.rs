use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use thiserror::Error;

use anyhow::Result;

enum Cell {
    Empty,
    Roll,
}

#[derive(Debug, Error)]
enum CellParseError {
    #[error("Invalid character in cell")]
    InvalidChar,
}

impl TryFrom<char> for Cell {
    type Error = CellParseError;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '.' => Ok(Cell::Empty),
            '@' => Ok(Cell::Roll),
            _ => Err(CellParseError::InvalidChar),
        }
    }
}

struct Grid {
    cells: Vec<Vec<Cell>>,
}

impl Grid {
    fn count_adjacent_rolls(&self, row: usize, col: usize) -> usize {
        let mut count = 0;
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dr, dc) in directions {
            let new_row = row as isize + dr;
            let new_col = col as isize + dc;

            if new_row >= 0
                && new_row < self.cells.len() as isize
                && new_col >= 0
                && new_col < self.cells[0].len() as isize
            {
                let new_row = new_row as usize;
                let new_col = new_col as usize;
                if let Some(cell) = self.get(new_row, new_col) {
                    if matches!(cell, Cell::Roll) {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn create_can_be_removed_count_grid(&self) -> Vec<Vec<bool>> {
        let mut count_grid = vec![vec![false; self.cols()]; self.rows()];

        for row in 0..self.rows() {
            for col in 0..self.cols() {
                if let Some(Cell::Roll) = self.get(row, col) {
                    let adjacent_rolls = self.count_adjacent_rolls(row, col);
                    count_grid[row][col] = adjacent_rolls < 4;
                }
            }
        }

        count_grid
    }

    fn count_can_be_removed(&self) -> usize {
        self.create_can_be_removed_count_grid()
            .iter()
            .flatten()
            .filter(|&&can_be_removed| can_be_removed)
            .count()
    }

    fn get(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cells.get(row)?.get(col)
    }

    fn rows(&self) -> usize {
        self.cells.len()
    }

    fn cols(&self) -> usize {
        self.cells.first().map_or(0, |row| row.len())
    }
}

fn parse_input(file: File) -> Result<Grid> {
    BufReader::new(file)
        .lines()
        .map(|line_res| {
            let line = line_res?;
            line.chars()
                .map(|ch| Cell::try_from(ch).map_err(|e| anyhow::anyhow!(e)))
                .collect::<Result<Vec<Cell>>>()
        })
        .collect::<Result<Vec<Vec<Cell>>>>()
        .map(|cells| Grid { cells })
}

fn part_1(grid: &Grid) -> usize {
    grid.count_can_be_removed()
}

fn part_2(mut grid: Grid) -> usize {
    let mut total_removed = 0;

    loop {
        let can_be_removed = grid.create_can_be_removed_count_grid();
        let can_be_removed_count = grid.count_can_be_removed();
        if can_be_removed_count == 0 {
            break;
        }
        total_removed += can_be_removed_count;

        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                if can_be_removed[row][col] {
                    grid.cells[row][col] = Cell::Empty;
                }
            }
        }
    }

    total_removed
}

fn main() -> Result<()> {
    let file = File::open("inputs/input04.txt")?;
    let grid = parse_input(file)?;

    println!("Part 1: {}", part_1(&grid));
    println!("Part 2: {}", part_2(grid));
    Ok(())
}
