use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub fn read_config_from_file() -> Result<(Value, Value), Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open("parameters.json")?;
    let reader = BufReader::new(file);

    let config: Value = serde_json::from_reader(reader)?;
    
    let wall_file_name = config["map"].as_str().unwrap();
    let wall_file = File::open("walls/".to_owned() + wall_file_name + ".json")?;
    let wall_reader = BufReader::new(wall_file);

    let wall_config: Value = serde_json::from_reader(wall_reader)?;

    // Return the `Config`.
    Ok((config, wall_config))
}