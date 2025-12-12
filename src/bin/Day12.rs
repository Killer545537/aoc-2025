use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid cell character: {0}")]
    InvalidCellChar(char),
    #[error("Invalid shape format")]
    InvalidShapeFormat,
    #[error("Invalid region format")]
    InvalidRegionFormat,
    #[error("Parse integer error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Clone)]
enum Cell {
    Empty,
    Occupied,
}

impl TryFrom<char> for Cell {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Cell::Empty),
            '#' => Ok(Cell::Occupied),
            c => Err(ParseError::InvalidCellChar(c)),
        }
    }
}

struct Shape {
    grid: Vec<Vec<Cell>>,
}

struct Region {
    length: usize,
    width: usize,
    quantity: Vec<usize>,
}

impl TryFrom<&str> for Shape {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines: Vec<&str> = value.trim().lines().collect();
        if lines.is_empty() {
            return Err(ParseError::InvalidShapeFormat);
        }

        let mut grid = Vec::new();
        for line in lines {
            let mut row = Vec::new();
            for ch in line.chars() {
                row.push(Cell::try_from(ch)?);
            }
            grid.push(row);
        }

        Ok(Shape { grid })
    }
}

fn parse_input(file: File) -> Result<(Vec<Shape>, Vec<Region>)> {
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    let mut shapes = Vec::new();
    let mut regions = Vec::new();

    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines
        if line.is_empty() {
            i += 1;
            continue;
        }

        if line.contains(':') && line.chars().next().unwrap_or(' ').is_ascii_digit() {
            if line.contains('x') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 2 {
                    return Err(ParseError::InvalidRegionFormat.into());
                }

                let dimensions: Vec<&str> = parts[0].trim().split('x').collect();
                if dimensions.len() != 2 {
                    return Err(ParseError::InvalidRegionFormat.into());
                }

                let width: usize = dimensions[0].parse()?;
                let length: usize = dimensions[1].parse()?;

                // Parse quantities "1 0 1 0 3 2"
                let quantity: Result<Vec<usize>, _> = parts[1]
                    .trim()
                    .split_whitespace()
                    .map(|s| s.parse())
                    .collect();

                let quantity = quantity?;

                regions.push(Region {
                    length,
                    width,
                    quantity,
                });

                i += 1;
            } else {
                let mut shape_lines = Vec::new();
                i += 1;

                while i < lines.len() {
                    let shape_line = lines[i].trim();
                    if shape_line.is_empty() {
                        break;
                    }
                    if shape_line.contains(':') {
                        break;
                    }
                    shape_lines.push(shape_line);
                    i += 1;
                }

                if shape_lines.is_empty() {
                    return Err(ParseError::InvalidShapeFormat.into());
                }

                let shape_str = shape_lines.join("\n");
                let shape = Shape::try_from(shape_str.as_str())?;
                shapes.push(shape);
            }
        } else {
            return Err(ParseError::InvalidShapeFormat.into());
        }
    }

    Ok((shapes, regions))
}

fn part_1(_shapes: &[Shape], regions: &[Region]) -> usize {
    regions
        .iter()
        .filter(|region| {
            let present_area: usize = region.quantity.iter().map(|&qty| qty * 8).sum();
            let region_area = region.length * region.width;
            present_area < region_area
        })
        .count()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input12.txt")?;
    let (shapes, regions) = parse_input(file)?;

    println!("Part 1: {}", part_1(&shapes, &regions));

    Ok(())
}
