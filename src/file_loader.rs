use crate::state::State;
use byteorder::{BigEndian, ReadBytesExt};
use std::{fs, io::BufReader};

pub fn load_file(filename: String, mut state: State) -> Result<State, std::io::Error> {
    let mut reader = BufReader::new(fs::File::open(filename)?);
    let mut address = usize::from(reader.read_u16::<BigEndian>()?);

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(instruction) => {
                state.memory[address] = instruction;
                address += 1;
            }
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(state)
                } else {
                    Err(e)
                };
            }
        }
    }
}
