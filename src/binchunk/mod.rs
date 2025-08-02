use binary_chunk::Prototype;
use reader::Reader;

pub mod binary_chunk;
mod reader;

pub fn undump(data: Vec<u8>) -> Prototype {
    let mut reader = Reader::new(data);
    reader.checkHeader();
    reader.readByte();
    reader.readProto(String::from(""))
}