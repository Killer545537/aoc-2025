use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

use thiserror::Error;

type Device = String;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "Invalid line format at line {line_number}: '{line}' - expected format 'device: connection1 connection2 ...'"
    )]
    InvalidLineFormat { line_number: usize, line: String },

    #[error("Empty device name at line {line_number}")]
    EmptyDeviceName { line_number: usize },

    #[error("Device name contains invalid characters at line {line_number}: '{device}'")]
    InvalidDeviceName { line_number: usize, device: String },

    #[error("No connections specified for device '{device}' at line {line_number}")]
    NoConnections { line_number: usize, device: String },

    #[error("Empty input file - no devices found")]
    EmptyInput,
}

struct Server {
    adjacency_list: HashMap<Device, Vec<Device>>,
}

impl Server {
    fn new() -> Self {
        Server {
            adjacency_list: HashMap::new(),
        }
    }

    fn add_connection(&mut self, from: Device, to: Device) {
        self.adjacency_list
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);
    }

    fn get_connections(&self, device: &str) -> Option<&Vec<Device>> {
        self.adjacency_list.get(device)
    }

    fn count_paths(&self, start: &str, end: &str) -> usize {
        let mut visited = HashSet::new();
        self.dfs_count_paths(start, end, &mut visited)
    }

    fn dfs_count_paths(&self, current: &str, target: &str, visited: &mut HashSet<String>) -> usize {
        if current == target {
            return 1;
        }

        visited.insert(current.to_string());

        let mut total_paths = 0;

        if let Some(connections) = self.get_connections(current) {
            for next_device in connections {
                if !visited.contains(next_device) {
                    total_paths += self.dfs_count_paths(next_device, target, visited);
                }
            }
        }

        visited.remove(current);

        total_paths
    }

    fn count_paths_through_waypoints(&self, start: &str, end: &str, waypoints: &[&str]) -> usize {
        let mut visited = HashSet::new();
        let required: HashSet<String> = waypoints.iter().map(|w| w.to_string()).collect();

        // (current_node, waypoints_bitmask)
        let mut cache: HashMap<(String, u32), usize> = HashMap::new();

        self.dfs_count_paths_with_waypoints(
            start,
            end,
            &mut visited,
            0,
            &required,
            waypoints,
            &mut cache,
        )
    }

    fn dfs_count_paths_with_waypoints(
        &self,
        current: &str,
        target: &str,
        visited: &mut HashSet<String>,
        waypoints_mask: u32,
        required: &HashSet<String>,
        waypoints_list: &[&str],
        cache: &mut HashMap<(String, u32), usize>,
    ) -> usize {
        if current == target {
            let all_visited = (1 << waypoints_list.len()) - 1;
            return if waypoints_mask == all_visited { 1 } else { 0 };
        }

        let cache_key = (current.to_string(), waypoints_mask);
        if let Some(&cached_result) = cache.get(&cache_key) {
            return cached_result;
        }

        visited.insert(current.to_string());

        let mut new_mask = waypoints_mask;
        if let Some(pos) = waypoints_list.iter().position(|&w| w == current) {
            new_mask |= 1 << pos;
        }

        let mut total_paths = 0;

        if let Some(connections) = self.get_connections(current) {
            for next_device in connections {
                if !visited.contains(next_device) {
                    total_paths += self.dfs_count_paths_with_waypoints(
                        next_device,
                        target,
                        visited,
                        new_mask,
                        required,
                        waypoints_list,
                        cache,
                    );
                }
            }
        }

        visited.remove(current);

        cache.insert(cache_key, total_paths);
        total_paths
    }
}

fn parse_input(file: File) -> Result<Server, ParseError> {
    let mut server = Server::new();
    let reader = BufReader::new(file);
    let mut line_number = 0;
    let mut has_devices = false;

    let is_valid_device_name = |name: &str| -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
    };

    for line in reader.lines() {
        let line = line?;
        line_number += 1;
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        let (from_device, connections) =
            trimmed_line
                .split_once(':')
                .ok_or_else(|| ParseError::InvalidLineFormat {
                    line_number,
                    line: line.clone(),
                })?;

        let from_device = from_device.trim();

        if from_device.is_empty() {
            return Err(ParseError::EmptyDeviceName { line_number });
        }

        if !is_valid_device_name(from_device) {
            return Err(ParseError::InvalidDeviceName {
                line_number,
                device: from_device.to_string(),
            });
        }

        let connections_str = connections.trim();

        if connections_str.is_empty() {
            return Err(ParseError::NoConnections {
                line_number,
                device: from_device.to_string(),
            });
        }

        for to_device in connections_str.split_whitespace() {
            if !is_valid_device_name(to_device) {
                return Err(ParseError::InvalidDeviceName {
                    line_number,
                    device: to_device.to_string(),
                });
            }

            server.add_connection(from_device.to_string(), to_device.to_string());
        }

        has_devices = true;
    }

    if !has_devices {
        return Err(ParseError::EmptyInput);
    }

    Ok(server)
}

fn part_1(server: &Server) -> usize {
    server.count_paths("you", "out")
}

fn part_2(server: &Server) -> usize {
    server.count_paths_through_waypoints("svr", "out", &["dac", "fft"])
}

fn main() -> Result<(), ParseError> {
    let file = File::open("inputs/input11.txt")?;
    let server = parse_input(file)?;

    println!("Part 1: {}", part_1(&server));
    println!("Part 2: {}", part_2(&server));

    Ok(())
}
