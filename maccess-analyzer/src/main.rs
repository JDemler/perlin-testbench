use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let stdinlock = stdin.lock();
    let mut last = 0;
    for line in stdinlock.lines() {        
        let line = line.unwrap();
        let z = u64::from_str_radix(&line[2..14], 16).unwrap();
        if last == 0 {
            last = z;
        } 
        println!("{:?}{}", z as i64 - last as i64, &line[14..]);
        last = z;
    }    
}
