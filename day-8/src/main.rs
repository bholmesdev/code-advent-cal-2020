use regex::Regex;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Debug)]
enum OP {
    Nop(i64),
    Acc(i64),
    Jmp(i64),
}

fn to_int(value: &str) -> i64 {
    match value.parse::<i64>() {
        Ok(i) => i,
        Err(_e) => 0,
    }
}

fn to_operation(raw_instruction: regex::Captures) -> OP {
    // ex: map "acc +1" -> OP::Acc
    let op_type = &raw_instruction[1];
    let num = to_int(&raw_instruction[2]);
    match op_type {
        "acc" => OP::Acc(num),
        "jmp" => OP::Jmp(num),
        _ => OP::Nop(num),
    }
}

fn jump_index(index: u32, change: i64) -> u32 {
    match change + index as i64 {
        index if index >= 0 => index as u32,
        _ => 0,
    }
}

fn nop_index(index: u32) -> u32 {
    index + 1
}

fn accumulate_from_instructions(
    instructions: &Vec<OP>,
    swap_op_index: Option<u32>,
) -> (i64, Option<u32>, HashMap<u32, u32>) {
    let mut visited_instruction_graph: HashMap<u32, u32> = HashMap::new();
    let mut acc = 0;
    let mut index = 0;

    loop {
        if let Some(_) = visited_instruction_graph.get(&index) {
            // if we've already been here before, it's an infinite loop!
            // break out with the index we looped back to
            break (acc, Some(index), visited_instruction_graph);
        } else if index >= instructions.len() as u32 {
            // if we exceed the length of the array, then we terminated successfully!
            // break out with the final acc value
            break (acc, None, visited_instruction_graph);
        } else {
            let next_index = match instructions[index as usize] {
                OP::Acc(num) => {
                    acc += num;
                    index + 1
                }
                OP::Jmp(num) => match swap_op_index {
                    // if we're attempting to "swap" a faulty instruction,
                    // switch to the nop behavior for this one
                    Some(swap_index) if swap_index == index => nop_index(index),
                    _ => jump_index(index, num),
                },
                OP::Nop(num) => match swap_op_index {
                    // vice versa for this faulty instruction
                    Some(swap_index) if swap_index == index => jump_index(index, num),
                    _ => nop_index(index),
                },
            };
            visited_instruction_graph.insert(index, next_index);
            index = next_index;
        }
    }
}

fn get_possibly_broken_instructions(
    visited_instruction_graph: &HashMap<u32, u32>,
    initial_index: u32,
) -> Vec<u32> {
    let mut index = initial_index;
    let mut possibly_broken_instructions: Vec<u32> = Vec::new();
    loop {
        let next_index = *visited_instruction_graph.get(&index).unwrap();
        possibly_broken_instructions.push(index);
        if next_index == initial_index {
            break; // we've closed the loop!
        } else {
            index = next_index;
        }
    }
    possibly_broken_instructions
}

fn accumulate_and_fix_broken_instruction(instructions: &Vec<OP>) -> i64 {
    let (acc, finished_early_at_index, visited_instruction_graph) =
        accumulate_from_instructions(instructions, None);
    match finished_early_at_index {
        // if we didn't finish early, we got it right the first try
        None => acc,
        Some(initial_index) => {
            let possibly_broken_instructions =
                get_possibly_broken_instructions(&visited_instruction_graph, initial_index);
            let mut index = 0;
            loop {
                let instruction_index = possibly_broken_instructions[index];
                match instructions[instruction_index as usize] {
                    // if it's an accumulation function, it couldn't *possibly* be the error
                    OP::Acc(_) => (),
                    // otherwise, let's try running the accumulator again
                    // swapping the Nop for Jmp (or vice versa)
                    _ => {
                        let (acc, finished_early_at_index, _) =
                            accumulate_from_instructions(instructions, Some(instruction_index));
                        match finished_early_at_index {
                            Some(_) => (),
                            // if we didn't finish early, we can break from the loop!
                            None => break acc,
                        }
                    }
                }
                index += 1;
            }
        }
    }
}

fn main() {
    let read_instructions = Regex::new(r"(acc|nop|jmp) ([\+|\-][0-9]+)").unwrap();
    let raw_input = read_to_string("instructions.txt");

    match raw_input {
        Ok(raw_input) => {
            let instructions: Vec<OP> = read_instructions
                // get all the capture groups we found
                .captures_iter(&raw_input)
                // map each group to a shiny enum we can work with
                .map(|instruction| to_operation(instruction))
                .collect();

            println!(
                "Our accumulator hit {}",
                accumulate_and_fix_broken_instruction(&instructions)
            );
        }
        Err(_) => println!("Something's wrong with the input file!"),
    }
}
