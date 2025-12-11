use anyhow::{Result, anyhow};
use std::{collections::HashSet, fs::File, io::Read};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Light {
    On,
    Off,
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("invalid character for Light: {0}")]
    InvalidLightCharacter(char),
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("missing light diagram")]
    MissingLightDiagram,
    #[error("missing joltage requirements")]
    MissingJoltageRequirements,
    #[error("invalid button wiring format")]
    InvalidButtonWiring,
    #[error("invalid format")]
    InvalidFormat,
}

impl TryFrom<char> for Light {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Light::On),
            '.' => Ok(Light::Off),
            c => Err(ParseError::InvalidLightCharacter(c)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct LightDiagram(Vec<Light>);

impl LightDiagram {
    fn new(size: usize) -> Self {
        LightDiagram(vec![Light::Off; size])
    }

    fn toggle(&mut self, button: usize) {
        if let Some(light) = self.0.get_mut(button) {
            *light = match light {
                Light::On => Light::Off,
                Light::Off => Light::On,
            };
        }
    }

    fn matches(&self, target: &LightDiagram) -> bool {
        self.0 == target.0
    }
}

impl TryFrom<&str> for LightDiagram {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value
            .chars()
            .map(Light::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(LightDiagram)
    }
}

type Button = u8;

#[derive(Debug, PartialEq, Eq)]
struct ButtonWiring(Vec<HashSet<Button>>);

impl TryFrom<&str> for ButtonWiring {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = value.trim();
        if s.is_empty() {
            return Ok(ButtonWiring(Vec::new()));
        }

        let groups = s
            .split_whitespace()
            .filter(|group| !group.is_empty())
            .map(|group| {
                if !group.starts_with('(') || !group.ends_with(')') {
                    return Err(ParseError::InvalidButtonWiring);
                }

                let inner = &group[1..group.len() - 1];
                if inner.is_empty() {
                    return Err(ParseError::InvalidButtonWiring);
                }

                inner
                    .split(',')
                    .map(|part| {
                        part.trim()
                            .parse::<u8>()
                            .map_err(|_| ParseError::InvalidNumber(part.to_string()))
                    })
                    .collect::<Result<HashSet<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        if groups.is_empty() {
            Err(ParseError::InvalidButtonWiring)
        } else {
            Ok(ButtonWiring(groups))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct JoltageRequirements(Vec<u8>);

impl TryFrom<&str> for JoltageRequirements {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = value.trim();
        let inner = if s.starts_with('{') && s.ends_with('}') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        if inner.trim().is_empty() {
            return Ok(JoltageRequirements(Vec::new()));
        }

        inner
            .split(',')
            .map(|part| {
                part.trim()
                    .parse::<u8>()
                    .map_err(|_| ParseError::InvalidNumber(part.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(JoltageRequirements)
    }
}

#[derive(Debug)]
struct Machine {
    light_diagram: LightDiagram,
    button_wiring: ButtonWiring,
    joltage_requirements: JoltageRequirements,
}

impl TryFrom<&str> for Machine {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = value.trim();

        let (start_ld, end_ld) = {
            let start = s.find('[').ok_or(ParseError::MissingLightDiagram)?;
            let end = s[start + 1..]
                .find(']')
                .ok_or(ParseError::MissingLightDiagram)?;
            (start, start + 1 + end)
        };

        let (start_j, end_j) = {
            let start = s.find('{').ok_or(ParseError::MissingJoltageRequirements)?;
            let end = s[start + 1..]
                .find('}')
                .ok_or(ParseError::MissingJoltageRequirements)?;
            (start, start + 1 + end)
        };

        if end_ld + 1 > start_j {
            return Err(ParseError::InvalidFormat);
        }

        let light_diagram = LightDiagram::try_from(&s[start_ld + 1..end_ld])?;
        let button_wiring = ButtonWiring::try_from(&s[end_ld + 1..start_j])?;
        let joltage_requirements = JoltageRequirements::try_from(&s[start_j + 1..end_j])?;

        Ok(Machine {
            light_diagram,
            button_wiring,
            joltage_requirements,
        })
    }
}

fn parse_input(mut file: File) -> Result<Vec<Machine>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(idx, line)| {
            Machine::try_from(line).map_err(|e| anyhow!("parse error at line {}: {}", idx + 1, e))
        })
        .collect()
}

fn min_toggles_recursive(
    current: &mut LightDiagram,
    target: &LightDiagram,
    button_groups: &[HashSet<Button>],
    group_index: usize,
) -> Option<usize> {
    if current.matches(target) {
        return Some(0);
    }

    if group_index >= button_groups.len() {
        return None;
    }

    let group = &button_groups[group_index];

    // Try not using this button group
    let without_group = min_toggles_recursive(current, target, button_groups, group_index + 1);

    // Try using this button group
    for &button in group {
        current.toggle(button as usize);
    }

    let with_group =
        min_toggles_recursive(current, target, button_groups, group_index + 1).map(|cost| cost + 1);

    // Undo the toggle
    for &button in group {
        current.toggle(button as usize);
    }

    match (without_group, with_group) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn min_toggles_joltage_only_recursive(
    current_joltage: &mut Vec<u8>,
    target_joltage: &JoltageRequirements,
    button_groups: &[HashSet<Button>],
    group_index: usize,
    memo: &mut std::collections::HashMap<(Vec<u8>, usize), Option<usize>>,
) -> Option<usize> {
    // Check memo
    let key = (current_joltage.clone(), group_index);
    if let Some(&result) = memo.get(&key) {
        return result;
    }

    // Early pruning: if any joltage exceeds target, this path is invalid
    for (current, &target) in current_joltage.iter().zip(target_joltage.0.iter()) {
        if *current > target {
            memo.insert(key, None);
            return None;
        }
    }

    if *current_joltage == target_joltage.0 {
        memo.insert(key, Some(0));
        return Some(0);
    }

    if group_index >= button_groups.len() {
        memo.insert(key, None);
        return None;
    }

    let group = &button_groups[group_index];
    let mut best_cost = None;

    // Try using this group 0, 1, 2, ... times (up to reasonable limit)
    let max_uses = target_joltage.0.iter().max().unwrap_or(&0) * 2;

    for uses in 0..=max_uses as usize {
        // Apply the group 'uses' times
        for _ in 0..uses {
            for &button in group {
                if let Some(jolt) = current_joltage.get_mut(button as usize) {
                    *jolt = jolt.saturating_add(1);
                }
            }
        }

        // Check if this path is still valid (early pruning)
        let mut valid = true;
        for (current, &target) in current_joltage.iter().zip(target_joltage.0.iter()) {
            if *current > target {
                valid = false;
                break;
            }
        }

        if valid {
            if let Some(cost) = min_toggles_joltage_only_recursive(
                current_joltage,
                target_joltage,
                button_groups,
                group_index + 1,
                memo,
            ) {
                let total_cost = cost + uses;
                best_cost = Some(best_cost.map_or(total_cost, |prev: usize| prev.min(total_cost)));
            }
        }

        // Undo the group applications
        for _ in 0..uses {
            for &button in group {
                if let Some(jolt) = current_joltage.get_mut(button as usize) {
                    if *jolt > 0 {
                        *jolt -= 1;
                    }
                }
            }
        }
    }

    memo.insert(key, best_cost);
    best_cost
}

fn solve_machine(machine: &Machine) -> Option<usize> {
    let target = &machine.light_diagram;
    let mut current = LightDiagram::new(target.0.len());
    let button_groups = &machine.button_wiring.0;

    min_toggles_recursive(&mut current, target, button_groups, 0)
}

fn solve_machine_joltage_only(machine: &Machine) -> Option<usize> {
    let target_joltage = &machine.joltage_requirements;
    let mut current_joltage = vec![0u8; target_joltage.0.len()];
    let button_groups = &machine.button_wiring.0;
    let mut memo = std::collections::HashMap::new();

    min_toggles_joltage_only_recursive(
        &mut current_joltage,
        target_joltage,
        button_groups,
        0,
        &mut memo,
    )
}

fn part_1(machines: &[Machine]) -> usize {
    machines
        .iter()
        .filter_map(|machine| solve_machine(machine))
        .sum()
}

fn part_2(machines: &[Machine]) -> usize {
    machines
        .iter()
        .filter_map(|machine| solve_machine_joltage_only(machine))
        .sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input10.txt")?;
    let machines = parse_input(file)?;

    println!("Part 1: {}", part_1(&machines));
    println!("Part 2: {}", part_2(&machines));

    Ok(())
}
