use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use thiserror::Error;

enum Cell {
    Start,
    Empty,
    Splitter,
}

#[derive(Debug, Error)]
enum CellParseError {
    #[error("Invalid cell character")]
    InvalidCell,
}

impl TryFrom<char> for Cell {
    type Error = CellParseError;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            'S' => Ok(Cell::Start),
            '.' => Ok(Cell::Empty),
            '^' => Ok(Cell::Splitter),
            _ => Err(CellParseError::InvalidCell),
        }
    }
}

struct Grid(Vec<Vec<Cell>>);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BeamState {
    x: usize,
    y: usize,
}

impl Grid {
    fn from_file(file: File) -> Result<Self> {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

        let grid: Vec<Vec<Cell>> = lines
            .iter()
            .map(|line| {
                line.chars()
                    .map(|ch| Cell::try_from(ch))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Grid(grid))
    }

    fn find_start(&self) -> Option<(usize, usize)> {
        for (row_idx, row) in self.0.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if matches!(cell, Cell::Start) {
                    return Some((col_idx, row_idx));
                }
            }
        }
        None
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.0.get(y)?.get(x)
    }

    fn simulate_beams(&self) -> Result<usize> {
        let start = self
            .find_start()
            .ok_or_else(|| anyhow::anyhow!("No start position found"))?;

        // Simple BFS
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut visited_splitters = HashSet::new();

        queue.push_back(BeamState {
            x: start.0,
            y: start.1,
        });

        while let Some(beam) = queue.pop_front() {
            if !visited.insert(beam) {
                continue;
            }

            // Move beam down until it hits a splitter or goes out of bounds
            let mut current_y = beam.y + 1;

            while current_y < self.0.len() {
                if let Some(cell) = self.get_cell(beam.x, current_y) {
                    if matches!(cell, Cell::Splitter) {
                        visited_splitters.insert((beam.x, current_y));

                        if beam.x > 0 {
                            queue.push_back(BeamState {
                                x: beam.x - 1,
                                y: current_y,
                            });
                        }

                        if beam.x + 1 < self.0.get(current_y).map_or(0, |row| row.len()) {
                            queue.push_back(BeamState {
                                x: beam.x + 1,
                                y: current_y,
                            });
                        }

                        break;
                    }
                }
                current_y += 1;
            }
        }

        Ok(visited_splitters.len())
    }

    fn count_leaf_timelines(&self) -> Result<usize> {
        let start = self
            .find_start()
            .ok_or_else(|| anyhow::anyhow!("No start position found"))?;

        let mut memo = HashMap::new();
        self.count_paths_from_position(start.0, start.1, &mut memo)
    }

    fn count_paths_from_position(
        &self,
        x: usize,
        y: usize,
        memo: &mut HashMap<(usize, usize), usize>,
    ) -> Result<usize> {
        if let Some(&cached) = memo.get(&(x, y)) {
            return Ok(cached);
        }

        let result = self.compute_paths_from_position(x, y, memo)?;
        memo.insert((x, y), result);
        Ok(result)
    }

    fn compute_paths_from_position(
        &self,
        x: usize,
        y: usize,
        memo: &mut HashMap<(usize, usize), usize>,
    ) -> Result<usize> {
        // Move down until we hit a splitter or reach the bottom
        let mut current_y = y + 1;

        while current_y < self.0.len() {
            if let Some(cell) = self.get_cell(x, current_y) {
                if matches!(cell, Cell::Splitter) {
                    // Hit a splitter - sum paths from left and right positions
                    let mut total = 0;

                    if x > 0 {
                        total += self.count_paths_from_position(x - 1, current_y, memo)?;
                    }

                    if x + 1 < self.0.get(current_y).map_or(0, |row| row.len()) {
                        total += self.count_paths_from_position(x + 1, current_y, memo)?;
                    }

                    return Ok(total);
                }
            }
            current_y += 1;
        }

        // Reached the bottom without hitting a splitter - this is a leaf path
        Ok(1)
    }
}

fn main() -> Result<()> {
    let file = File::open("inputs/input07.txt")?;
    let grid = Grid::from_file(file)?;

    let split_count = grid.simulate_beams()?;
    let leaf_timeline_count = grid.count_leaf_timelines()?;

    println!("Number of splits: {}", split_count);
    println!("Number of leaf timelines: {}", leaf_timeline_count);

    Ok(())
}
