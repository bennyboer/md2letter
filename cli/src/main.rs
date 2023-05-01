use md2letter_convert::convert;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let result = convert(Box::new(reader))?;

    println!("{}", result);

    Ok(())
}
