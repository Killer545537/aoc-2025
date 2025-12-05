use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
};

use anyhow::Result;
use thiserror::Error;

struct Range {
    start: u128,
    end: u128,
}

impl Range {
    fn contains(&self, value: u128) -> bool {
        value >= self.start && value <= self.end
    }
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

        let start = parts[0].parse()?;
        let end = parts[1].parse()?;

        Ok(Range { start, end })
    }
}

type Ingredient = u128;
type FreshIngredients = Vec<Range>;

fn parse_input(file: File) -> Result<(FreshIngredients, Vec<Ingredient>)> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut fresh_ingredients = FreshIngredients::new();

    for line_result in &mut lines {
        let line = line_result?;
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        let range = Range::try_from(line)?;
        fresh_ingredients.push(range);
    }

    let mut ingredients = Vec::new();
    for line_result in lines {
        let line = line_result?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let ingredient: Ingredient = line.parse()?;
        ingredients.push(ingredient);
    }

    Ok((fresh_ingredients, ingredients))
}

fn part_1(fresh_ingredients: &FreshIngredients, ingredients: &[Ingredient]) -> usize {
    ingredients
        .iter()
        .filter(|&&ingredient| {
            fresh_ingredients
                .iter()
                .any(|range| range.contains(ingredient))
        })
        .count()
}

fn part_2(mut fresh_ingredients: FreshIngredients) -> u128 {
    if fresh_ingredients.is_empty() {
        return 0;
    }

    fresh_ingredients.sort_by_key(|r| r.start);
    let mut total = 0;
    let mut current_end = 0;

    for range in fresh_ingredients {
        if range.start > current_end {
            total += range.end - range.start + 1;
            current_end = range.end;
        } else if range.end > current_end {
            total += range.end - current_end;
            current_end = range.end;
        }
    }

    total
}

fn main() -> Result<()> {
    let file = File::open("inputs/input05.txt")?;
    let (fresh_ingredients, ingredients) = parse_input(file)?;

    println!("Part 1: {}", part_1(&fresh_ingredients, &ingredients));
    println!("Part 2: {}", part_2(fresh_ingredients));
    Ok(())
}
