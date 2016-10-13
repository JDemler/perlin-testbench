extern crate perlin;
extern crate time;

use std::env;
use std::fs::File;
use std::io::{Bytes, Read, Write};
use std::iter::Peekable;

use perlin::storage::RamStorage;
use perlin::index::boolean_index::{BooleanIndex, IndexBuilder};

macro_rules! try_option{
    ($operand:expr) => {
        if let Some(x) = $operand {
            x
        } else {
            return None;
        }
    }
}


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
        Err(e) => {
            println!("Unable to open file {}:{}", args[1].clone(), e);
            return;
        }
    };

    let mut bytes = Vec::new();
    collection_file.read_to_end(&mut bytes).unwrap();

    let collection = CollectionIterator::new(VByteDecoder::new(bytes.bytes()));
    let docs = collection.docs;
    let len = collection.len;
    let start = time::PreciseTime::now();
    let index = index(collection);
    println!("");
    println!("DONE! Indexed {} documents each {} terms totalling at  {} in {}ms",
             docs,
             len,
             fmt_bytes(docs * len * 8),
             start.to(time::PreciseTime::now()).num_milliseconds());
    println!("At a rate of {}/s",
             fmt_bytes((docs * len * 8 * 1000) /
                       start.to(time::PreciseTime::now()).num_milliseconds() as usize));
    println!("{}", index.document_count());


}

fn index<R: Read>(collection: CollectionIterator<R>) -> BooleanIndex<usize>{
    IndexBuilder::<_, RamStorage<_>>::new()
        .create(collection.map(|v| v.into_iter()))
        .unwrap()
 
}

fn fmt_bytes(bytes: usize) -> String {
    let mut factor = 1_000_000_000_000;
    let mut unit = "Tb";
    if bytes < 1_000_000_000_000 {
        factor = 1_000_000_000;
        unit = "Gb";
    }
    if bytes < 1_000_000_000 {
        factor = 1_000_000;
        unit = "Mb";
    }
    if bytes < 1_000_000 {
        factor = 1_000;
        unit = "Kb";
    }
    if bytes < 1_000 {
        factor = 1;
        unit = "b";
    }
    format!("{}{}", bytes / factor, unit)
}

pub struct CollectionIterator<R> {
    iter: VByteDecoder<R>,
    docs: usize,
    len: usize,
    pos: usize,
}

impl<'a, R: Read + 'a> CollectionIterator<R> {
    fn new(mut decoder: VByteDecoder<R>) -> Self {
        let docs = decoder.next().unwrap();
        let len = decoder.next().unwrap();
        CollectionIterator {
            iter: decoder,
            docs: docs,
            len: len,
            pos: 0,
        }
    }
}

impl<R: Read> Iterator for CollectionIterator<R> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        if self.pos <= self.docs {
            return Some((&mut self.iter).take(self.len).collect::<Vec<_>>());
        }
        None
    }
}

/// Iterator that decodes a bytestream to unsigned integers
pub struct VByteDecoder<R> {
    bytes: Bytes<R>,
}

impl<R: Read> VByteDecoder<R> {
    /// Create a new VByteDecoder by passing a bytestream
    pub fn new(read: Bytes<R>) -> Self {
        VByteDecoder { bytes: read }
    }

    /// Sometimes it is convenient to look at the original bytestream itself
    /// (e.g. when not only vbyte encoded integers are in the bytestream)
    /// This method provides access to the underlying bytestream in form of
    /// a
    /// mutable borrow
    pub fn underlying_iterator(&mut self) -> &mut Bytes<R> {
        &mut self.bytes
    }
}

impl<R: Read> Iterator for VByteDecoder<R> {
    type Item = usize;

    /// Returns the next unsigned integer which is encoded in the underlying
    /// bytestream
    /// May iterate the underlying bytestream an arbitrary number of times
    /// Returns None when the underlying bytream returns None
    fn next(&mut self) -> Option<Self::Item> {

        let mut result: usize = 0;
        loop {
            result *= 128;
            let val = try_option!(self.bytes.next()).unwrap();
            result += val as usize;
            if val >= 128 {
                result -= 128;
                break;
            }
        }
        Some(result)
    }
}
