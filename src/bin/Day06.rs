use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
enum Operator {
    Plus,
    Multiply,
}

#[derive(Debug, Error)]
enum ParseOperatorError {
    #[error("Invalid operator")]
    InvalidOperator,
}

impl TryFrom<&str> for Operator {
    type Error = ParseOperatorError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operator::Plus),
            "*" => Ok(Operator::Multiply),
            _ => Err(ParseOperatorError::InvalidOperator),
        }
    }
}

#[derive(Debug)]
struct ColumnData {
    numbers: Vec<u128>,
    operation: Operator,
}

impl ColumnData {
    fn compute_result(&self) -> u128 {
        match self.operation {
            Operator::Plus => self.numbers.iter().sum(),
            Operator::Multiply => self.numbers.iter().product(),
        }
    }
}

fn parse_input(file: File) -> Result<Vec<ColumnData>> {
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    let (op_line, data_lines) = lines
        .split_last()
        .ok_or_else(|| anyhow::anyhow!("Input file is empty"))?;

    let operations: Vec<Operator> = op_line
        .split_whitespace()
        .map(|op_str| Operator::try_from(op_str))
        .collect::<Result<Vec<_>, _>>()?;

    let grid: Vec<Vec<u128>> = data_lines
        .iter()
        .map(|line| {
            line.split_whitespace()
                .map(|num_str| num_str.parse())
                .collect()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let num_cols = grid.first().map_or(0, |row| row.len());
    (0..num_cols)
        .map(|col| {
            let numbers = grid.iter().map(|row| row[col]).collect();
            let operation = operations[col];
            Ok(ColumnData { numbers, operation })
        })
        .collect()
}

fn parse_input_right_to_left(file: File) -> Result<Vec<ColumnData>> {
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    // Convert to character grid
    let grid: Vec<Vec<char>> = lines.iter().map(|line| line.chars().collect()).collect();

    // Transpose to get columns
    let max_len = grid.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut cols = Vec::new();

    for col_idx in 0..max_len {
        let mut col = Vec::new();
        for row in &grid {
            if col_idx < row.len() {
                col.push(row[col_idx]);
            } else {
                col.push(' ');
            }
        }
        cols.push(col);
    }

    // Group columns by space-only columns
    let mut groups = Vec::new();
    let mut current_group = Vec::new();

    for col in cols {
        if col.iter().all(|&c| c == ' ') {
            if !current_group.is_empty() {
                groups.push(current_group);
                current_group = Vec::new();
            }
        } else {
            current_group.push(col);
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    // Convert groups to ColumnData
    let mut result = Vec::new();
    for group in groups {
        if group.is_empty() {
            continue;
        }

        // Get the operator from the last character of the first column
        let operator_char = *group[0]
            .last()
            .ok_or_else(|| anyhow::anyhow!("Empty column"))?;
        let operator = match operator_char {
            '+' => Operator::Plus,
            '*' => Operator::Multiply,
            _ => return Err(anyhow::anyhow!("Invalid operator: {}", operator_char)),
        };

        // Build numbers from each column (excluding the last character)
        let mut numbers = Vec::new();
        for col in &group {
            let number_chars: String = col[..col.len().saturating_sub(1)].iter().collect();
            let number_str = number_chars.trim();
            if !number_str.is_empty() {
                numbers.push(number_str.parse::<u128>()?);
            }
        }

        result.push(ColumnData {
            numbers,
            operation: operator,
        });
    }

    Ok(result)
}

fn part_1(columns: Vec<ColumnData>) -> u128 {
    columns.iter().map(|col| col.compute_result()).sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input06.txt")?;
    let columns = parse_input(file)?;
    println!("Part 1: {}", part_1(columns));

    let file = File::open("inputs/input06.txt")?;
    let columns = parse_input_right_to_left(file)?;
    println!("Part 2: {}", part_1(columns));

    Ok(())
}
