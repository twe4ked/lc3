use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Error};

pub fn read_rom(filename: String) -> Result<Vec<u16>, Error> {
    let mut reader = BufReader::new(File::open(filename)?);
    let mut buffer = Vec::new();

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(value) => {
                buffer.push(value);
            }
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(buffer)
                } else {
                    Err(e)
                };
            }
        }
    }
}
