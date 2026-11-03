extern crate flate2;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env::args;
use std::fs::File;
use std::io::copy;
use std::process;
use std::time::Instant;
use std::io::BufReader;


fn main() {
    if args().len() != 3 {
        eprintln!("Usage: `source` `target`");
        process::exit(1);
    }
    let source_path = args().nth(1).unwrap();
    let file = match File::open(&source_path) {
        Ok(f)=>f,
        Err(_)=> {
            eprintln!("Error: The file '{}' could not be found.", source_path);
            process::exit(1);
        }
    };
    let mut input = BufReader::new(file);
    let mut target_path = args().nth(2).unwrap();
    if !target_path.ends_with(".pdf.gz") {
        target_path.push_str(".pdf.gz");
    }
    let output = File::create(&target_path).unwrap();
    let mut encoder = GzEncoder::new(output, Compression::default());
    let start = Instant::now();
    copy(&mut input, &mut encoder ).unwrap();
    let output = encoder.finish().unwrap();
    println!(
        "Source len: {:?}",
        input.get_ref().metadata().unwrap().len()
    );
    println!("Target len: {:?}",output.metadata().unwrap().len());
    println!("Elapsed: {:?}",start.elapsed());
}
