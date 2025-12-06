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

fn part_1(columns: Vec<ColumnData>) -> u128 {
    columns.iter().map(|col| col.compute_result()).sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input06.txt")?;
    let columns = parse_input(file)?;

    println!("Part 1: {}", part_1(columns));

    let file2 = File::open("inputs/input06.txt")?;
    let cephalopod_columns = parse_right_to_left(file2)?;

    println!("\nCephalopod columns:");
    for (i, col) in cephalopod_columns.iter().enumerate() {
        println!(
            "Column {}: numbers {:?}, op {:?}, result = {}",
            i,
            col.numbers,
            col.operation,
            col.compute_result()
        );
    }

    println!("Part 2: {}", part_1(cephalopod_columns));
    Ok(())
}
