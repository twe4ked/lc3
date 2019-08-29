use byteorder::{BigEndian, ByteOrder};
use std::fs::File;
use std::io::{Error, Read};

pub fn read_rom(filename: String) -> Result<Vec<u16>, Error> {
    let mut data = Vec::new();
    File::open(filename)?.read_to_end(&mut data)?;

    assert!(data.len() % 2 == 0, "invalid ROM");

    let mut buffer = vec![0; data.len() / 2];
    BigEndian::read_u16_into(&data, &mut buffer);

    Ok(buffer)
}
