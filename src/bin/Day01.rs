use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
struct Rotation {
    direction: Direction,
    amount: i32,
}

#[derive(Debug, Error)]
enum DirectionError {
    #[error("Invalid direction character: '{0}'. Expected 'L' or 'R'.")]
    InvalidCharacter(char),
    #[error("Input string is empty.")]
    EmptyString,
}

#[derive(Debug, Error)]
enum RotationError {
    #[error("Direction parsing error: {0}")]
    Direction(#[from] DirectionError),
    #[error("Amount parsing error: {0}")]
    Amount(#[from] ParseIntError),
    #[error("String too short: '{0}'. Expected format like 'L25' or 'R1'")]
    TooShort(String),
    #[error("Amount must be non-negative, got: {0}")]
    NegativeAmount(i32),
}

impl TryFrom<&str> for Direction {
    type Error = DirectionError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let value = value.trim();

        if value.is_empty() {
            return Err(DirectionError::EmptyString);
        }

        match value.chars().next().unwrap().to_ascii_uppercase() {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            c => Err(DirectionError::InvalidCharacter(c)),
        }
    }
}

impl TryFrom<&str> for Rotation {
    type Error = RotationError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let value = value.trim();

        if value.is_empty() {
            return Err(RotationError::TooShort(value.to_string()));
        }

        if value.len() < 2 {
            return Err(RotationError::TooShort(value.to_string()));
        }

        // Get the first character for direction
        let direction_str = &value[0..1];
        let direction = Direction::try_from(direction_str)?;

        // Get the rest for amount
        let amount_str = &value[1..];
        let amount: i32 = amount_str.parse()?;

        if amount < 0 {
            return Err(RotationError::NegativeAmount(amount));
        }

        Ok(Rotation { direction, amount })
    }
}

fn read_rotations(file: File) -> Result<Vec<Rotation>> {
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.map_err(Into::into)) // Convert io::Error to anyhow::Error
        .filter_map(|line_result| {
            match line_result {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        None // Skip empty lines
                    } else {
                        Some(Ok(line.to_string()))
                    }
                }
                Err(e) => Some(Err(e)), // Propagate IO errors
            }
        })
        .map(|line_result| {
            line_result.and_then(|line| {
                Rotation::try_from(line.as_str())
                    .map_err(|e| anyhow!("Failed to parse rotation from '{}': {}", line, e))
            })
        })
        .collect()
}

fn part_1(position: i32, rotations: &[Rotation]) -> i32 {
    rotations
        .iter()
        .map(|rotation| match rotation.direction {
            Direction::Left => -rotation.amount,
            Direction::Right => rotation.amount,
        })
        .scan(position, |pos, signed_rotation| {
            *pos = (*pos + signed_rotation + 100) % 100;
            Some(*pos)
        })
        .filter(|&pos| pos == 0)
        .count() as i32
}

fn part_2(position: i32, rotations: &[Rotation]) -> i32 {
    rotations
        .iter()
        .map(|rotation| match rotation.direction {
            Direction::Left => -rotation.amount,
            Direction::Right => rotation.amount,
        })
        .scan(position, |pos, signed_rotation| {
            let old_pos = *pos;
            let new_pos_raw = old_pos + signed_rotation;

            // Check if we crossed the 0 boundary
            let crossed_zero = if signed_rotation > 0 {
                // Going right: crossed if we went from < 100 to >= 100
                old_pos < 100 && new_pos_raw >= 100
            } else if signed_rotation < 0 {
                // Going left: crossed if we went from >= 0 to < 0
                old_pos >= 0 && new_pos_raw < 0
            } else {
                false
            };

            // Normalize position for next iteration
            *pos = ((new_pos_raw % 100) + 100) % 100;

            Some(crossed_zero) // Always return Some to continue iteration
        })
        .filter(|&crossed| crossed)
        .count() as i32
}

fn main() -> Result<()> {
    let file = File::open("inputs/input01.txt")?;
    let rotations = read_rotations(file)?;

    println!("Part 1: {}", part_1(50, &rotations));
    println!("Part 2: {}", part_2(50, &rotations));

    Ok(())
}
