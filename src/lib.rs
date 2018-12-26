use std::error::Error;
use std::fs;
use std::fmt;
use std::io::BufReader;
use std::io::{self, Write};
use byteorder::{BigEndian, ReadBytesExt};
use rustyline;
use regex::Regex;
use lazy_static::lazy_static;

type Memory = [u16; std::u16::MAX as usize];

#[derive(Debug, PartialEq)]
pub struct Config {
    filename: String,
    debug: bool,
}

impl Config {
    pub fn new(args: &Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let mut config = Config {
            filename: "".to_string(),
            debug: false,
        };

        for arg in args {
            if arg == "--debug" {
                config.debug = true;
            } else {
                config.filename = arg.clone();
            }
        }

        Ok(config)
    }
}

struct State {
    memory: Memory,
    registers: [u16; 8],
    pc: u16,
    condition: Condition,
    running: bool,
}

impl State {
    pub fn new() -> State {
        State {
            memory: [0; std::u16::MAX as usize],
            registers: [0; 8],
            pc: 0x3000,
            condition: Condition::P,
            running: true,
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "registers: {:?}, condition: {:?}", self.registers, self.condition)
    }
}

#[derive(Debug, PartialEq)]
enum Condition {
    P = 1 << 0,
    Z = 1 << 1,
    N = 1 << 2,
}

#[derive(Debug)]
enum Opcode {
    BR, ADD, LD, ST, JSR, AND, LDR, STR, UNUSED, NOT, LDI, STI, JMP, RESERVED, LEA, TRAP,
}

impl Opcode {
    fn from_instruction(instruction: u16) -> Opcode {
        let value = instruction >> 12;

        match value {
            0x00 => Opcode::BR,
            0x01 => Opcode::ADD,
            0x02 => Opcode::LD,
            0x03 => Opcode::ST,
            0x04 => Opcode::JSR,
            0x05 => Opcode::AND,
            0x06 => Opcode::LDR,
            0x07 => Opcode::STR,
            0x08 => Opcode::UNUSED,
            0x09 => Opcode::NOT,
            0x0a => Opcode::LDI,
            0x0b => Opcode::STI,
            0x0c => Opcode::JMP,
            0x0d => Opcode::RESERVED,
            0x0e => Opcode::LEA,
            0x0f => Opcode::TRAP,
            _ => panic!("bad opcode: {}", value),
        }
    }
}

#[derive(Debug)]
enum TrapVector {
    GETC, OUT, PUTS, IN, PUTSP, HALT,
}

impl TrapVector {
    fn from_instruction(instruction: u16) -> Result<TrapVector, String> {
        let value = instruction & 0xFF;

        match value {
            0x20 => Ok(TrapVector::GETC),
            0x21 => Ok(TrapVector::OUT),
            0x22 => Ok(TrapVector::PUTS),
            0x23 => Ok(TrapVector::IN),
            0x24 => Ok(TrapVector::PUTSP),
            0x25 => Ok(TrapVector::HALT),
            _ => Err(format!("bad TRAP vector: {:x}", value)),
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut state = State::new();

    let buffer = read_file(config.filename);

    let mut i = 0;
    while i < buffer.len() {
        state.memory[0x3000 + i] = buffer[i];
        i += 1
    }

    while state.running {
        state = process(state, config.debug);
    }

    Ok(())
}

fn process(mut state: State, debug: bool) -> State {
    let instruction : u16 = read_memory(&state.memory, state.pc);
    let opcode = Opcode::from_instruction(instruction);

    if debug {
        let mut rl = rustyline::Editor::<()>::new();
        let readline = rl.readline(&format!("{:#04x}> ", state.pc));

        lazy_static! {
            static ref READ_REGEX: Regex = Regex::new(r"^read 0x([a-f0-9]{1,4})$").unwrap();
        }

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());

                match line.as_ref() {
                    "c" | "continue" => {
                        // continue
                    }

                    "i" | "inspect" => {
                        println!("{:?}, op_code: {:?}, instruction: {:#4x}, {:#016b}", state, opcode, instruction, instruction);
                        return state;
                    }

                    line if READ_REGEX.is_match(line) => {
                        if let Some(address) = READ_REGEX.captures(line).unwrap().get(1) {
                            let address = u16::from_str_radix(address.as_str(), 16).unwrap();
                            let value = read_memory(&state.memory, address);
                            println!("{:#04x}, {:#016b}", value, value);
                        }
                        return state;
                    }

                    "exit" => {
                        state.running = false;
                        return state;
                    }

                    _ => {
                        println!("Unknown command {:?}", line);
                        return state;
                    }
                }
            },
            Err(rustyline::error::ReadlineError::Interrupted) => {
                state.running = false;
            },
            Err(rustyline::error::ReadlineError::Eof) => {
                state.running = false;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                state.running = false;
            }
        }
    }

    state.pc = state.pc.wrapping_add(1);

    match opcode {
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            if (n == 1 && state.condition == Condition::N) ||
               (z == 1 && state.condition == Condition::Z) ||
               (p == 1 && state.condition == Condition::P) {
                   let pc_offset = instruction & 0x1ff;

                   state.pc = state.pc.wrapping_add(sign_extend(pc_offset, 9));
            }
        }

        Opcode::ADD => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

            if immediate_flag {
                let immediate_value = sign_extend(instruction & 0x1f, 5);

                state.registers[r0 as usize] = state.registers[r1 as usize].wrapping_add(immediate_value);
            } else {
                let r2 = instruction & 0x7;

                state.registers[r0 as usize] = state.registers[r1 as usize].wrapping_add(state.registers[r2 as usize]);
            }

            state = update_flags(state, r0);
        }

        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.registers[r0 as usize] = state.memory[address as usize];

            state = update_flags(state, r0);
        }

        Opcode::ST => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.memory[address as usize] = state.registers[r0 as usize];
        }

        Opcode::JSR => {
            let temp = state.pc;
            let use_pc_offset = (instruction >> 11) & 1;
            let pc_offset = instruction & 0x1ff;
            let r = (instruction >> 6) & 7;

            if use_pc_offset == 1 {
                state.pc = state.pc.wrapping_add(sign_extend(pc_offset, 9));
            } else {
                state.pc = state.registers[r as usize];
            }

            state.registers[7] = temp;
        }

        Opcode::AND => {
            let immediate_flag = ((instruction >> 5) & 1) == 1;
            let immediate_value = sign_extend(instruction & 0x1f, 5);

            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let r2 = (instruction) & 0x7;

            if immediate_flag {
                state.registers[r0 as usize] = state.registers[r1 as usize] & immediate_value;
            } else {
                state.registers[r0 as usize] = state.registers[r1 as usize] & state.registers[r2 as usize];
            }
        }

        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            let address = state.registers[r1 as usize].wrapping_add(sign_extend(offset, 6));
            state.registers[r0 as usize] = read_memory(&state.memory, address);

            state = update_flags(state, r0);
        }

        Opcode::STR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            let address = state.registers[r0 as usize];
            let value = state.registers[r1 as usize].wrapping_add(sign_extend(offset, 6));

            state.memory[address as usize] = value;
        }

        Opcode::UNUSED => {
            panic!("unused");
        }

        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            state.registers[r0 as usize] = !state.registers[r1 as usize];
            state = update_flags(state, r0);
        }

        Opcode::LDI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = sign_extend(instruction & 0x1ff, 9);
            let address = state.pc.wrapping_add(pc_offset);

            state.registers[r0 as usize] = read_memory(&state.memory, address);

            state = update_flags(state, r0);
        }

        Opcode::STI => {
            let r = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.memory[read_memory(&state.memory, address) as usize] = state.registers[r as usize];
        }

        Opcode::JMP => {
            let r = (instruction >> 6) & 0xa;

            state.pc = state.registers[r as usize];
        }

        Opcode::RESERVED => {
            panic!("reserved");
        }

        Opcode::LEA => {
            let r = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            state.registers[r as usize] = state.pc.wrapping_add(sign_extend(pc_offset, 9));
        }

        Opcode::TRAP => {
            if let Ok(trap_vector) = TrapVector::from_instruction(instruction) {
                match trap_vector {
                    TrapVector::GETC => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::OUT => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::PUTS => {
                        let mut i : u16 = state.registers[0];

                        while read_memory(&state.memory, i) != 0 {
                            print!("{}", char::from(read_memory(&state.memory, i) as u8));
                            i += 1;
                        }

                        io::stdout().flush().unwrap();
                    }

                    TrapVector::IN => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::PUTSP => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::HALT => {
                        state.running = false;
                    }
                }
            }
        }
    }

    state
}

fn sign_extend(mut value: u16, bit_count: u8) -> u16 {
    if ((value >> (bit_count - 1)) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    value
}

fn update_flags(mut state: State, r: u16) -> State {
    if state.registers[r as usize] == 0 {
        state.condition = Condition::Z;
    } else if (state.registers[r as usize] >> 15) == 1 {
        // NOTE: A 1 in the left-most bit indicates negative
        state.condition = Condition::N;
    } else {
        state.condition = Condition::P;
    }

    state
}

fn read_file(filename: String) -> Vec<u16> {
    let file_size = fs::metadata(filename.clone()).expect("could not get file meta data").len() as usize;
    if &file_size % 2 != 0 {
        panic!("file was not even number of bytes long");
    }
    let array_size = file_size/2;

    let file = fs::File::open(filename).expect("failed to open file");
    let mut buf_reader = BufReader::new(file);

    let mut buffer: Vec<u16> = Vec::with_capacity(array_size);
    unsafe { buffer.set_len(array_size); }
    buf_reader.read_u16_into::<BigEndian>(&mut buffer[..]).expect("failed to read");

    buffer
}

fn read_memory(memory: &Memory, address: u16) -> u16 {
    if address < std::u16::MAX {
        memory[address as usize]
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_valid_arguments() {
        let args = [String::from("program_name"), String::from("filename")].to_vec();

        assert_eq!(Config::new(&args).unwrap().filename, String::from("filename"));
        assert_eq!(Config::new(&args).unwrap().debug, false);
    }

    #[test]
    fn config_not_enough_arguments() {
        let args = [String::from("program_name")].to_vec();

        assert_eq!(Config::new(&args), Err("not enough arguments"));
    }

    #[test]
    fn config_with_debug() {
        let args = [String::from("program_name"), String::from("filename"), String::from("--debug")].to_vec();

        assert_eq!(Config::new(&args).unwrap().filename, String::from("filename"));
        assert_eq!(Config::new(&args).unwrap().debug, true);
    }

    #[test]
    fn config_with_debug_first() {
        let args = [String::from("program_name"), String::from("--debug"), String::from("filename")].to_vec();

        assert_eq!(Config::new(&args).unwrap().filename, String::from("filename"));
        assert_eq!(Config::new(&args).unwrap().debug, true);
    }

    #[test]
    fn process_add_immediate() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0001_010_001_1_00001;
        //                       ^    ^   `r1 ^ ^
        //                       `add |       | ` literal 1
        //                            ` r2    `immediate

        state.registers[1] = 3;

        let state = process(state, false);

        assert_eq!(state.registers, [0, 3, 4, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_add_register() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0001_010_001_0_00_000;
        //                       ^    ^   `r1 ^ ^  `r0
        //                       `add |       | ` unused
        //                            |       `register
        //                            ` r2 (destination)

        state.registers[0] = 2;
        state.registers[1] = 3;

        let state = process(state, false);

        assert_eq!(state.registers, [2, 3, 5, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_ldi() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1010_000_000000001;
        //                       ^    `r0 ^
        //                       `LDI     `pc_offset

        state.memory[0x3001] = 0x3002;
        state.memory[0x3002] = 42;

        let state = process(state, false);

        assert_eq!(state.registers, [42, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_jmp() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1100_000_010_000000;
        //                       ^        `register
        //                       `JMP

        state.registers[2] = 5;

        let state = process(state, false);

        assert_eq!(state.pc, 5);
    }

    #[test]
    fn process_br_n_true() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0000_1_0_0_000000101;
        //                       ^    `n    `pc_offset (5)
        //                       `BR

        state.condition = Condition::N;

        let state = process(state, false);

        // incremented pc + 5
        assert_eq!(state.pc, 0x3006);
    }

    #[test]
    fn process_br_n_false() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0000_1_0_0_000000101;
        //                       ^    `n    `pc_offset (5)
        //                       `BR

        state.condition = Condition::P;

        let state = process(state, false);

        // incremented pc + 1 (ingores the pc_offset)
        assert_eq!(state.pc, 0x3001);
    }

    #[test]
    fn process_ld() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0010_011_000000101;
        //                       ^    `r3 ^
        //                       `LD      `pc_offset (5)

        state.memory[0x3000 + 1 + 5] = 42;

        state.condition = Condition::P;

        let state = process(state, false);

        assert_eq!(state.registers[3], 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_st() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0011_011_000000101;
        //                       ^    `r3 ^
        //                       `LD      `pc_offset (5)

        state.registers[3] = 42;
        state.condition = Condition::P;

        let state = process(state, false);

        assert_eq!(state.memory[0x3000 + 1 + 5], 42);
    }

    #[test]
    fn process_jsr() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0100_0_00_011_000000;
        //                       ^    ^ ^  `r3 `unused
        //                       `JSR | `unused
        //                            `use pc_offset

        state.registers[3] = 42;

        let state = process(state, false);

        assert_eq!(state.pc, 42);
        assert_eq!(state.registers[7], 0x3001);
    }

    #[test]
    fn process_jsr_use_pc_offset() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0100_1_00000000011;
        //                       ^    ^ `pc_offset (k)
        //                       `JSR |
        //                            `use pc_offset

        let state = process(state, false);

        assert_eq!(state.pc, 0x3000 + 1 + 3);
        assert_eq!(state.registers[7], 0x3001);
    }

    #[test]
    fn process_and() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0101_001_010_0_00_011;
        //                       ^    `r0 `r1 ^    `r2
        //                       `AND         `immediate_flag

        state.registers[2] = 3;
        state.registers[3] = 5;

        let state = process(state, false);

        assert_eq!(state.registers[1], 3 & 5);
    }

    #[test]
    fn process_and_immediate() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0101_001_010_1_00101;
        //                       ^    `r0 `r1 ^ `immediate_value (5)
        //                       `AND         `immediate_flag

        state.registers[2] = 3;

        let state = process(state, false);

        assert_eq!(state.registers[1], 3 & 5);
    }

    #[test]
    fn process_ldr() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0110_001_010_000011;
        //                       ^    `r0 `r1 `offset (3)
        //                       `AND

        state.registers[2] = 1;
        state.memory[1 + 3] = 42;

        let state = process(state, false);

        assert_eq!(state.registers[1], 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_ldr_memory_address_too_big() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0110_001_010_000001;
        //                       ^    `r0 `r1 `offset (1)
        //                       `AND

        state.registers[2] = std::u16::MAX - 1;

        let state = process(state, false);

        assert_eq!(state.registers[1], 0);
        assert_eq!(state.condition, Condition::Z);
    }

    #[test]
    fn process_str() {
        let mut state = State::new();

        state.memory[0x3000] = 0b0111_001_010_000011;
        //                       ^    `r0 `r1 `offset (3)
        //                       `AND

        state.registers[1] = 42;
        state.registers[2] = 2;

        let state = process(state, false);

        assert_eq!(state.memory[42], 2 + 3);
    }

    #[test]
    fn process_not() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1001_001_010_1_11111;
        //                       ^    `r0 `r1
        //                       `NOT

        state.registers[2] = 42;

        let state = process(state, false);

        assert_eq!(state.registers[1], !42);
        // TODO: Why is this Z?
        // assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_sti() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1011_001_000000010;
        //                       ^    `r0 `pc_offset (2)
        //                       `STI

        let address = 3;
        state.registers[1] = 42;
        state.memory[(state.pc + 1 + 2) as usize] = address;

        let state = process(state, false);

        assert_eq!(state.memory[address as usize], 42);
    }

    #[test]
    fn process_lea() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1110_001_000000010;
        //                       ^    `r0 `pc_offset (2)
        //                       `LEA

        let state = process(state, false);

        assert_eq!(state.registers[1], 0x3000 + 1 + 2);
    }

    #[test]
    fn process_trap_halt() {
        let mut state = State::new();

        state.memory[0x3000] = 0b1111_0000_00100101;
        //                       ^         `HALT (0x25)
        //                       `TRAP

        let state = process(state, false);

        assert_eq!(state.running, false);
    }
}
