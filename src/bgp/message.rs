use std::io::Write;

pub struct MessageHeader {}

impl MessageHeader {
    pub fn to_bytes(self) -> Result<Vec<u8>, anyhow::Error> {
        println!("XXX to_bytes");
        let buf: Vec<u8> = Vec::new();
        let mut c = std::io::Cursor::new(buf);
        c.write(&vec![0xff; 16])?;
        //c.write_u8(10u8)?;
        let vec = c.into_inner();
        println!("XXX to_bytes vec.len {}", vec.len());

        Ok(vec)
    }
}
