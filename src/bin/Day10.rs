use anyhow::{Result, anyhow};
use good_lp::{Expression, Solution, SolverModel, Variable, default_solver, variable, variables};
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

fn solve_machine(machine: &Machine) -> Option<usize> {
    let target = &machine.light_diagram;
    let mut current = LightDiagram::new(target.0.len());
    let button_groups = &machine.button_wiring.0;

    min_toggles_recursive(&mut current, target, button_groups, 0)
}

fn solve_machine_joltage(machine: &Machine) -> Option<usize> {
    let target = &machine.joltage_requirements.0;
    let button_groups = &machine.button_wiring.0;

    if target.is_empty() {
        return Some(0);
    }

    // Check if problem is feasible
    for (pos, &target_joltage) in target.iter().enumerate() {
        let mut any_button_affects_pos = false;
        for group in button_groups.iter() {
            if group.contains(&(pos as u8)) {
                any_button_affects_pos = true;
                break;
            }
        }
        if !any_button_affects_pos && target_joltage > 0 {
            return None;
        }
    }

    // Create variable per button (number of times it got pressed)
    let mut vars = variables!();
    let presses: Vec<Variable> = (0..button_groups.len())
        .map(|_| vars.add(variable().min(0).integer()))
        .collect();

    // Minimize total presses
    let total_presses: Expression = presses.iter().sum();
    let mut problem = vars.minimise(total_presses).using(default_solver);

    // For each jolt counter, sum of relevant presses must equal the target joltage
    for (jolt_idx, &target_joltage) in target.iter().enumerate() {
        let mut expr = Expression::from(0.0);

        for (btn_idx, relevant_idxs) in button_groups.iter().enumerate() {
            // If button is relevant, add its press variable to the constraint
            if relevant_idxs.contains(&(jolt_idx as u8)) {
                expr += presses[btn_idx];
            }
        }

        // Sum of relevant presses == target joltage
        problem.add_constraint(expr.eq(target_joltage as f64));
    }

    match problem.solve() {
        Ok(solution) => {
            let total = presses
                .iter()
                .map(|v| solution.value(*v).round() as usize)
                .sum();
            Some(total)
        }
        Err(_) => None,
    }
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
        .filter_map(|machine| solve_machine_joltage(machine))
        .sum()
}

fn main() -> Result<()> {
    let file = File::open("inputs/input10.txt")?;
    let machines = parse_input(file)?;

    println!("Part 1: {}", part_1(&machines));
    println!("Part 2: {}", part_2(&machines));

    Ok(())
}
