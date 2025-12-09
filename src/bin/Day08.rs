use std::{cmp::Ordering, collections::HashSet, fs::File};

use anyhow::Result;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

impl Coordinate {
    fn distance(&self, other: &Coordinate) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        let dz = (self.z - other.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl TryFrom<&str> for Coordinate {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid coordinate format"));
        }
        let x = parts[0].trim().parse::<i32>()?;
        let y = parts[1].trim().parse::<i32>()?;
        let z = parts[2].trim().parse::<i32>()?;
        Ok(Coordinate { x, y, z })
    }
}

struct DSU {
    parent: Vec<usize>,
}

impl DSU {
    fn new(n: usize) -> Self {
        DSU {
            parent: (0..n).collect(),
        }
    }

    fn root(&mut self, x: usize) -> usize {
        if self.parent[x] == x {
            return x;
        }
        self.parent[x] = self.root(self.parent[x]);
        self.parent[x]
    }

    fn merge(&mut self, a: usize, b: usize) {
        let root_a = self.root(a);
        let root_b = self.root(b);
        self.parent[root_a] = root_b;
    }

    fn count_components(&mut self, n: usize) -> usize {
        let mut unique_roots = HashSet::new();
        for i in 0..n {
            unique_roots.insert(self.root(i));
        }
        unique_roots.len()
    }
}

fn parse_input(file: File) -> Result<Vec<Coordinate>> {
    use std::io::{BufRead, BufReader};

    BufReader::new(file)
        .lines()
        .map(|line| {
            line.map_err(Into::into)
                .and_then(|l| Coordinate::try_from(l.as_str()))
        })
        .collect()
}

fn build_circuits(coordinates: &[Coordinate], num_edges: usize) -> Vec<usize> {
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    for i in 0..coordinates.len() {
        for j in (i + 1)..coordinates.len() {
            let distance = coordinates[i].distance(&coordinates[j]);
            edges.push((i, j, distance));
        }
    }

    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));

    let mut dsu = DSU::new(coordinates.len());

    let num_edges = edges.len().min(num_edges);
    for &(a, b, _) in &edges[..num_edges] {
        dsu.merge(a, b);
    }

    let mut sizes = vec![0; coordinates.len()];
    for i in 0..coordinates.len() {
        let root = dsu.root(i);
        sizes[root] += 1;
    }

    sizes.sort_by(|a, b| b.cmp(a));

    sizes
}

fn part_1(circuit_sizes: &[usize]) -> usize {
    assert!(circuit_sizes.len() >= 3);

    circuit_sizes[0] * circuit_sizes[1] * circuit_sizes[2]
}

fn part_2(coordinates: &[Coordinate]) -> i32 {
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    for i in 0..coordinates.len() {
        for j in (i + 1)..coordinates.len() {
            let distance = coordinates[i].distance(&coordinates[j]);
            edges.push((i, j, distance));
        }
    }

    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));

    let mut dsu = DSU::new(coordinates.len());

    for &(a, b, _) in &edges {
        if dsu.root(a) != dsu.root(b) {
            dsu.merge(a, b);

            if dsu.count_components(coordinates.len()) == 1 {
                return coordinates[a].x * coordinates[b].x;
            }
        }
    }

    unreachable!("All coordinates should be connected");
}

fn main() -> Result<()> {
    let file = File::open("inputs/input08.txt")?;
    let coordinates = parse_input(file)?;

    let circuit_sizes = build_circuits(&coordinates, 1000);
    println!("Part 1: {}", part_1(&circuit_sizes));
    println!("Part 2: {}", part_2(&coordinates));

    Ok(())
}
