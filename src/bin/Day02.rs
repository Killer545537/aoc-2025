use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use thiserror::Error;

struct Range {
    start: u128,
    end: u128,
}

#[derive(Debug, Error)]
enum RangeParseError {
    #[error("Range is missing a '-' separator")]
    MissingDash,

    #[error("Invalid number: {0}")]
    InvalidInt(#[from] ParseIntError),
}

impl TryFrom<&str> for Range {
    type Error = RangeParseError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<_> = value.split('-').collect();
        if parts.len() != 2 {
            return Err(RangeParseError::MissingDash);
        }

        let start: u128 = parts[0].parse()?;
        let end: u128 = parts[1].parse()?;

        Ok(Range { start, end })
    }
}

fn parse_ranges_line(file: File) -> Result<Vec<Range>> {
    BufReader::new(file)
        .lines()
        .next()
        .ok_or_else(|| anyhow!("File is empty"))?
        .map_err(|e| anyhow!("Failed to read line: {}", e))?
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            Range::try_from(s).map_err(|e| anyhow!("Failed to parse rotation from: {}: {}", s, e))
        })
        .collect()
}

fn part_1(ranges: &[Range]) -> u128 {
    let is_invalid_id = |id: String| {
        if id.len() % 2 != 0 {
            return false;
        }
        let (first_half, second_half) = id.split_at(id.len() / 2);
        first_half == second_half
    };

    ranges
        .into_iter()
        .map(|range| {
            let Range { start, end } = range;

            (*start..=*end).filter(|&id| {
                let string_id = id.to_string();
                is_invalid_id(string_id)
            })
        })
        .flatten()
        .sum()
}

fn part_2(ranges: &[Range]) -> u128 {
    let is_invalid_id = |id: String| {
        let idid = format!("{id}{id}");

        idid[1..idid.len() - 1].contains(id.as_str())
    };

    ranges
        .into_iter()
        .map(|range| {
            let Range { start, end } = range;

            (*start..=*end).filter(|&id| {
                let string_id = id.to_string();
                is_invalid_id(string_id)
            })
        })
        .flatten()
        .sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input02.txt")?;
    let ranges = parse_ranges_line(file)?;

    println!("Part 1: {}", part_1(&ranges));
    println!("Part 2: {}", part_2(&ranges));

    Ok(())
}
