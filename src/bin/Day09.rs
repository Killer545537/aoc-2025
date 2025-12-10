use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;

use anyhow::Result;
use itertools::Itertools;
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: i128,
    y: i128,
}

impl Coordinate {
    fn area_between(&self, other: &Coordinate) -> i128 {
        let width = (other.x - self.x).abs() + 1;
        let height = (other.y - self.y).abs() + 1;
        width * height
    }
}

struct CompressedGrid {
    grid: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl CompressedGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![0u8; height]; width],
            width,
            height,
        }
    }

    fn mark_cell(&mut self, x: usize, y: usize) {
        self.grid[x][y] = 1;
    }

    fn is_marked(&self, x: usize, y: usize) -> bool {
        self.grid[x][y] == 1
    }

    fn mark_all_interior(&mut self, outside: &HashSet<(i32, i32)>) {
        for x in 0..self.width {
            for y in 0..self.height {
                if !outside.contains(&(x as i32, y as i32)) {
                    self.grid[x][y] = 1;
                }
            }
        }
    }
}

struct PrefixSumArray {
    sums: Vec<Vec<i128>>,
}

impl PrefixSumArray {
    fn from_grid(grid: &CompressedGrid) -> Self {
        let mut sums = vec![vec![0i128; grid.height]; grid.width];

        for x in 0..grid.width {
            for y in 0..grid.height {
                let left = if x > 0 { sums[x - 1][y] } else { 0 };
                let top = if y > 0 { sums[x][y - 1] } else { 0 };
                let top_left = if x > 0 && y > 0 {
                    sums[x - 1][y - 1]
                } else {
                    0
                };
                sums[x][y] = left + top - top_left + grid.grid[x][y] as i128;
            }
        }

        Self { sums }
    }

    fn rectangle_sum(&self, min_x: usize, min_y: usize, max_x: usize, max_y: usize) -> i128 {
        let left = if min_x > 0 {
            self.sums[min_x - 1][max_y]
        } else {
            0
        };
        let top = if min_y > 0 {
            self.sums[max_x][min_y - 1]
        } else {
            0
        };
        let top_left = if min_x > 0 && min_y > 0 {
            self.sums[min_x - 1][min_y - 1]
        } else {
            0
        };

        self.sums[max_x][max_y] - left - top + top_left
    }
}

struct CoordinateMapper {
    x_map: HashMap<i128, usize>,
    y_map: HashMap<i128, usize>,
}

impl CoordinateMapper {
    fn new(coordinates: &[Coordinate]) -> Self {
        let create_mapping = |values: Vec<i128>| -> HashMap<i128, usize> {
            let mut sorted_values = values;
            sorted_values.sort_unstable();
            sorted_values.dedup();
            sorted_values
                .iter()
                .enumerate()
                .map(|(i, &val)| (val, i))
                .collect()
        };

        let xs = coordinates.iter().map(|c| c.x).collect();
        let ys = coordinates.iter().map(|c| c.y).collect();

        Self {
            x_map: create_mapping(xs),
            y_map: create_mapping(ys),
        }
    }

    fn map_coordinate(&self, coord: &Coordinate) -> (usize, usize) {
        (self.x_map[&coord.x] * 2, self.y_map[&coord.y] * 2)
    }

    fn grid_dimensions(&self) -> (usize, usize) {
        (self.x_map.len() * 2 - 1, self.y_map.len() * 2 - 1)
    }
}

#[derive(Error, Debug)]
enum ParseCoordinateError {
    #[error("Invalid format for coordinates")]
    InvalidFormat,
    #[error("Non-numeric value found in coordinates")]
    NonNumericValue,
}

impl TryFrom<&str> for Coordinate {
    type Error = ParseCoordinateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 2 {
            return Err(ParseCoordinateError::InvalidFormat);
        }

        let x = parts[0]
            .trim()
            .parse::<i128>()
            .map_err(|_| ParseCoordinateError::NonNumericValue)?;
        let y = parts[1]
            .trim()
            .parse::<i128>()
            .map_err(|_| ParseCoordinateError::NonNumericValue)?;

        Ok(Coordinate { x, y })
    }
}

fn parse_input(file: File) -> Result<Vec<Coordinate>> {
    use std::io::{BufRead, BufReader};

    let reader = BufReader::new(file);
    let mut coordinates = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let coord = Coordinate::try_from(line.as_str())?;
        coordinates.push(coord);
    }

    Ok(coordinates)
}

fn part_1(coordinates: &[Coordinate]) -> i128 {
    coordinates
        .iter()
        .combinations(2)
        .map(|pair| pair[0].area_between(pair[1]))
        .max()
        .unwrap_or(0)
}

fn mark_polygon_edges(
    grid: &mut CompressedGrid,
    coordinates: &[Coordinate],
    mapper: &CoordinateMapper,
) {
    for i in 0..coordinates.len() {
        let current = &coordinates[i];
        let next = &coordinates[(i + 1) % coordinates.len()];

        let (cx1, cy1) = mapper.map_coordinate(current);
        let (cx2, cy2) = mapper.map_coordinate(next);

        let (min_cx, max_cx) = if cx1 < cx2 { (cx1, cx2) } else { (cx2, cx1) };
        let (min_cy, max_cy) = if cy1 < cy2 { (cy1, cy2) } else { (cy2, cy1) };

        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                grid.mark_cell(cx, cy);
            }
        }
    }
}

fn flood_fill_exterior(grid: &CompressedGrid) -> HashSet<(i32, i32)> {
    let mut outside = HashSet::new();
    let mut queue = VecDeque::new();

    outside.insert((-1i32, -1i32));
    queue.push_back((-1i32, -1i32));

    let try_visit_neighbor = |nx: i32,
                              ny: i32,
                              grid: &CompressedGrid,
                              outside: &mut HashSet<(i32, i32)>,
                              queue: &mut VecDeque<(i32, i32)>| {
        // Skip if out of expanded bounds
        if nx < -1 || ny < -1 || nx > grid.width as i32 || ny > grid.height as i32 {
            return;
        }

        // Skip if hitting a wall (polygon edge)
        if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
            if grid.is_marked(nx as usize, ny as usize) {
                return;
            }
        }

        // Skip if already visited
        if outside.contains(&(nx, ny)) {
            return;
        }

        outside.insert((nx, ny));
        queue.push_back((nx, ny));
    };

    while let Some((tx, ty)) = queue.pop_front() {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = tx + dx;
            let ny = ty + dy;
            try_visit_neighbor(nx, ny, grid, &mut outside, &mut queue);
        }
    }

    outside
}

fn is_valid_rectangle(
    coord1: &Coordinate,
    coord2: &Coordinate,
    mapper: &CoordinateMapper,
    prefix_sum: &PrefixSumArray,
) -> bool {
    let (cx1, cy1) = mapper.map_coordinate(coord1);
    let (cx2, cy2) = mapper.map_coordinate(coord2);

    let (min_cx, max_cx) = if cx1 < cx2 { (cx1, cx2) } else { (cx2, cx1) };
    let (min_cy, max_cy) = if cy1 < cy2 { (cy1, cy2) } else { (cy2, cy1) };

    let count = prefix_sum.rectangle_sum(min_cx, min_cy, max_cx, max_cy);
    let expected = (max_cx - min_cx + 1) as i128 * (max_cy - min_cy + 1) as i128;

    count == expected
}

fn find_max_valid_area(
    coordinates: &[Coordinate],
    mapper: &CoordinateMapper,
    prefix_sum: &PrefixSumArray,
) -> i128 {
    let mut max_area = 0;

    for i in 0..coordinates.len() {
        for j in 0..i {
            let coord1 = &coordinates[i];
            let coord2 = &coordinates[j];

            if is_valid_rectangle(coord1, coord2, mapper, prefix_sum) {
                let area = coord1.area_between(coord2);
                max_area = max_area.max(area);
            }
        }
    }

    max_area
}

fn part_2(coordinates: &[Coordinate]) -> i128 {
    let mapper = CoordinateMapper::new(coordinates);
    let (grid_width, grid_height) = mapper.grid_dimensions();
    let mut grid = CompressedGrid::new(grid_width, grid_height);

    mark_polygon_edges(&mut grid, coordinates, &mapper);
    let outside = flood_fill_exterior(&grid);
    grid.mark_all_interior(&outside);

    let prefix_sum = PrefixSumArray::from_grid(&grid);
    find_max_valid_area(coordinates, &mapper, &prefix_sum)
}

fn main() -> Result<()> {
    let file = File::open("inputs/input09.txt")?;
    let coordinates = parse_input(file)?;

    println!("Part 1: {}", part_1(&coordinates));
    println!("Part 2: {}", part_2(&coordinates));

    Ok(())
}
