use crate::opcode::Opcode;
use crate::trap_vector::TrapVector;
use crate::utilities::sign_extend;

pub(crate) fn disassemble(instruction: u16, opcode: Opcode) {
    print!("{:?}, {:08b}_{:08b}, ", opcode, (instruction >> 8) & 0xff, instruction & 0xff);

    match opcode {
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            println!("n: {}, z: {}, p: {}", n, z, p);
        }

        Opcode::ADD => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

            println!("r0: {}, r1: {}, immediate_flag: {}", r0, r1, immediate_flag);
        }

        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("r0: {}, pc_offset: {}", r0, pc_offset);
        }

        Opcode::ST => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("r0: {}, pc_offset: {}", r0, pc_offset);
        }

        Opcode::JSR => {
            let use_pc_offset = (instruction >> 11) & 1;

            if use_pc_offset == 1 {
                let pc_offset = instruction & 0x1ff;

                println!("pc_offset: {}", pc_offset);
            } else {
                let r0 = (instruction >> 6) & 7;

                println!("(JSRR), r0: {}", r0);
            }
        }

        Opcode::AND => {
            let immediate_flag = ((instruction >> 5) & 1) == 1;
            let immediate_value = sign_extend(instruction & 0x1f, 5);

            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let r2 = (instruction) & 0x7;

            println!("immediate_flag: {}, immediate_value: {}, r0: {}, r1: {}, r2: {}",
                     immediate_flag, immediate_value, r0, r1, r2);
        }

        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            println!("r0: {}, r1: {}, offset: {}", r0, r1, offset);
        }

        Opcode::STR => {
            let sr = (instruction >> 9) & 0x7;
            let base_r = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            println!("sr: {:#04x}, base_r: {:#04x}, offset: {:#04x}", sr, base_r, offset);
        }

        Opcode::UNUSED => {
            panic!("unused");
        }

        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            println!("r0: {}, r1: {}", r0, r1);
        }

        Opcode::LDI => {
            let dr = (instruction >> 9) & 0x7;
            let pc_offset = sign_extend(instruction & 0x1ff, 9);

            println!("dr: {}, pc_offset: {}", dr, pc_offset);
        }

        Opcode::STI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("r0: {}, pc_offset: {}", r0, pc_offset);
        }

        Opcode::JMP => {
            let r0 = (instruction >> 6) & 0xa;

            println!("r0: {}", r0);
        }

        Opcode::RESERVED => {
            panic!("reserved");
        }

        Opcode::LEA => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("r0: {}, pc_offset: {}", r0, pc_offset);
        }

        Opcode::TRAP => {
            if let Ok(trap_vector) = TrapVector::from_instruction(instruction) {
                println!("trap_vector: {:?}", trap_vector);
            }
        }
    }
}
