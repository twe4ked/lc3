use std::fs::File;
use std::io::{Error, ErrorKind, Read};

pub fn read_rom(filename: String) -> Result<Vec<u16>, Error> {
    let mut data = Vec::new();
    File::open(filename)?.read_to_end(&mut data)?;
    from_bytes(&data)
}

fn from_bytes(data: &[u8]) -> Result<Vec<u16>, Error> {
    if data.len() % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "input must be a multiple of 2",
        ));
    }

    Ok(data
        .chunks(2)
        .map(|x| x[1] as u16 | (x[0] as u16) << 8)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let data = from_bytes(&vec![0xaa, 0xbb, 0xcc, 0xdd]).unwrap();
        assert_eq!(data, vec![0xaabb, 0xccdd]);

        let result = from_bytes(&vec![0xaa, 0xbb, 0xcc]).map_err(|e| e.kind());
        let expected = Err(ErrorKind::InvalidData);
        assert_eq!(result, expected);
    }
}
