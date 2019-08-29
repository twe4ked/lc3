use crate::state::State;
use byteorder::{BigEndian, ReadBytesExt};
use std::{fs, io::BufReader};

pub fn load_file(filename: String, mut state: State) -> Result<State, std::io::Error> {
    let mut reader = BufReader::new(fs::File::open(filename)?);
    let mut address = u16::from(reader.read_u16::<BigEndian>()?);

    state.pc = address;

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(value) => {
                state.memory.write(address, value);
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
