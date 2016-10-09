extern crate perlin;
extern crate time;

use std::env;
use std::fs::File;
use std::io::{Bytes, Read, Write};
use std::iter::Peekable;

use perlin::storage::RamStorage;
use perlin::index::boolean_index::IndexBuilder;

fn print_usage(program: &str) {
    let brief = format!("Usage: {} [COLLECTION_FILE]", program);
    println!("{}", brief);

}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let program = args[0].clone();
    if args.len() != 2 {
        print_usage(&program);
        return;
    }

    let mut collection_file = match File::open(args[1].clone()) {
        Ok(f) => f,
        Err(e) => { println!("Unable to open file {}:{}", args[1].clone(), e); return;}
    };

    // let mut bytes = Vec::new();
    // collection_file.read_to_end(&mut bytes).unwrap();

    let collection = CollectionIterator::new(collection_file.bytes());
    let mut c = 0;
    let index = IndexBuilder::<_, RamStorage<_>>::new().create(collection.inspect(|_| {
        c += 1;
        print!("\r{}", c);
        std::io::stdout().flush();
    })).unwrap();
    println!("DONE!");
}

pub struct CollectionIterator<R>
{
    iter: Bytes<R>,
}

impl<'a, R: Read + 'a> CollectionIterator<R> {
    fn new(decoder: Bytes<R>) -> Self {
        CollectionIterator{
            iter: decoder,
        }
    }
}

impl<R: Read> Iterator for CollectionIterator<R> {
    type Item = VByteDecoder;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = (&mut self.iter).map(|b| b.unwrap()).take_while(|p| *p != 0).collect::<Vec<_>>();        
        if bytes.is_empty() {         
            return None;
        }
        println!("{:?}", bytes);
        Some(VByteDecoder::new(bytes))        
    }
}

/// Iterator that decodes a bytestream to unsigned integers
pub struct VByteDecoder {
    bytes: Vec<u8>,
    pos: usize
}

impl VByteDecoder {
    /// Create a new VByteDecoder by passing a bytestream
    pub fn new(read: Vec<u8>) -> Self {
        VByteDecoder { bytes:  read, pos: 0 }
    }
}

impl Iterator for VByteDecoder {
    type Item = usize;

    /// Returns the next unsigned integer which is encoded in the underlying
    /// bytestream
    /// May iterate the underlying bytestream an arbitrary number of times
    /// Returns None when the underlying bytream returns None
    fn next(&mut self) -> Option<Self::Item> {

        let mut result: usize = 0;
        loop {
            result *= 128;
            let val = *match self.bytes.get(self.pos) {
                Some(byte) =>
                {
                    self.pos += 1;
                    byte                       
                },
                None => return None
            };
            if val == 0 {
                return None;
            }
            result += val as usize;
            if val >= 128 {
                result -= 128;
                break;
            }
        }
        Some(result)
    }
}
