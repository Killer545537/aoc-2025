use anyhow::{Error, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use thiserror::Error;

type Battery = u128;

struct Bank(Vec<Battery>);

impl Bank {
    fn largest_joltage_using_two(&self) -> u128 {
        let batteries = &self.0;
        let (best, _) =
            batteries
                .iter()
                .skip(1)
                .fold((0, batteries[0]), |(best, max_prev), &battery| {
                    let candidate = 10 * max_prev + battery;
                    let new_best = best.max(candidate);
                    let new_max_prev = max_prev.max(battery);
                    (new_best, new_max_prev)
                });

        best
    }

    /// Basically maximum subsequence of length k
    fn largest_joltage_using_k(&self, k: usize) -> u128 {
        let batteries = &self.0;
        let mut stack = Vec::new();
        let mut drop = batteries.len() - k;

        for battery in batteries {
            while drop > 0 && !stack.is_empty() && *stack.last().unwrap() < battery {
                stack.pop();
                drop -= 1;
            }
            stack.push(battery);
        }

        stack.truncate(k);

        stack.iter().fold(0u128, |acc, &d| acc * 10 + d)
    }
}

#[derive(Debug, Error)]
enum BankParseError {
    #[error("Invalid number: {0}")]
    InvalidInt(#[from] ParseIntError),
}

impl TryFrom<&str> for Bank {
    type Error = BankParseError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let batteries: Result<Vec<Battery>, ParseIntError> =
            value.chars().map(|c| c.to_string().parse()).collect();

        Ok(Bank(batteries?))
    }
}

fn parse_banks(file: File) -> Result<Vec<Bank>> {
    BufReader::new(file)
        .lines()
        .map(|line| {
            let line = line?;
            Bank::try_from(line.as_str()).map_err(Error::from)
        })
        .collect()
}

fn part_1(banks: &[Bank]) -> u128 {
    banks
        .iter()
        .map(|bank| bank.largest_joltage_using_two())
        .sum()
}

fn part_2(banks: &[Bank]) -> u128 {
    banks
        .iter()
        .map(|bank| bank.largest_joltage_using_k(12))
        .sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input03.txt")?;
    let banks = parse_banks(file)?;

    println!("Part 1: {}", part_1(&banks));
    println!("Part 2: {}", part_2(&banks));

    Ok(())
}
