extern crate rand;
extern crate getopts;

use std::io::Write;
use std::fs::File;
use std::env;

use getopts::Options;

use rand::{XorShiftRng, Rng};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [OUTPUT_FILE]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.reqopt("d", "docs", "Number of generated documents", "N");
    opts.reqopt("l", "length", "Length of the generated documents", "N");
    opts.optopt("v", "voc", "Size of the collection vocabulary", "N");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };
    let docs = matches.opt_str("d").map(|d| d.parse::<usize>().ok()).unwrap_or(None);
    let len = matches.opt_str("l").map(|d| d.parse::<usize>().ok()).unwrap_or(None);
    let voc = matches.opt_str("v").map(|d| d.parse::<usize>().ok()).unwrap_or(None);
    if matches.opt_present("h") || docs.is_none() || len.is_none() || matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }
    let file = File::create(matches.free[0].clone()).unwrap();
    let docs = docs.unwrap();
    let len = len.unwrap();
    let voc = voc.unwrap_or(voc_size(45, 0.5, docs * len));
    println!("Generating a collection with {} documents each {} terms long. This collection \
              contains {} total terms and sums up to {} in total writing the results in {}",
             docs,
             len,
             voc,
             fmt_bytes(docs * len * 8),
             matches.free[0].clone());
    println!("Continue? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if !input.starts_with("y") {
        return;
    }
    generate_collection(docs, len, voc, file);

}

fn generate_collection<W: Write>(docs: usize, len: usize, voc: usize, mut output: W) {

    let mut rng = ZipfGenerator::new(voc);
    for i in 0..docs {
        output.write(&rng.take(len)
                .map(|t| vbyte_encode(t))
                .flat_map(|tbytes| tbytes)
                .collect::<Vec<_>>())
            .unwrap();
        output.write(&[0]).unwrap();
        print!("\r{} of {} documents generated!", i + 1, docs);
        std::io::stdout().flush().unwrap();
    }
    println!("\nDone!");
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

// Implementation of Heaps' Law
fn voc_size(k: usize, b: f64, tokens: usize) -> usize {
    ((k as f64) * (tokens as f64).powf(b)) as usize
}

/// Encode an usigned integer as a variable number of bytes
pub fn vbyte_encode(mut number: usize) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        result.insert(0, (number % 128) as u8);
        if number < 128 {
            break;
        } else {
            number /= 128;
        }
    }
    let len = result.len();
    result[len - 1] += 128;
    result
}

#[derive(Clone)]
pub struct ZipfGenerator {
    voc_size: usize,
    factor: f64,
    acc_probs: Vec<f64>,
    rng: XorShiftRng,
}

impl ZipfGenerator {
    pub fn new(voc_size: usize) -> Self {
        let mut res = ZipfGenerator {
            voc_size: voc_size,
            factor: (1.78 * voc_size as f64).ln(),
            acc_probs: Vec::with_capacity(voc_size),
            rng: rand::weak_rng(),
        };
        let mut acc = 0.0;
        for i in 1..voc_size {
            acc += 1.0 / (i as f64 * res.factor);
            res.acc_probs.push(acc);
        }
        res
    }
}

impl<'a> Iterator for &'a mut ZipfGenerator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let dice = self.rng.gen::<f64>();
        let mut c = 0;
        loop {
            if dice < self.acc_probs[c] {
                return Some(c);
            }
            c += 1;
        }
    }
}
