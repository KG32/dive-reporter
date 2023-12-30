use std::io::{self, Read};
use std::error::Error;
use std::fs;
use serde::Deserialize;

static DEV_FILE_PATH: &str = "/Users/kubagroblewski/Documents/dive-reporter-tmp/Perdix 2[A76240BD]#61_2023-10-28.uddf";

struct DataPoint {
    time: String,
    depth: String,
}

#[derive(Debug, Deserialize)]
struct Document {
    #[serde(rename = "profiledata")]
    profile_data: Profiledata,
}

#[derive(Debug, Deserialize)]
struct Profiledata {
    #[serde(rename = "repetitiongroup")]
    repetition_group: RepetitionGroup,
}

#[derive(Debug, Deserialize)]
struct RepetitionGroup {
    dive: Dive,
}

#[derive(Debug, Deserialize)]
struct Dive {
    samples: Vec<Sample>,
}

#[derive(Debug, Deserialize)]
struct Sample {
    waypoint: Vec<Waypoint>,
}

#[derive(Debug, Deserialize)]
struct Waypoint {
    #[serde(rename = "divetime")]
    dive_time: u64,
    depth: f32,
}


pub fn parse_file() -> Result<(), Box<dyn Error>> {
    println!("Parse file");
    let tmp_file_path = DEV_FILE_PATH;
    let file_content = read_file_content(tmp_file_path)?;
    construct_document(&file_content);

    Ok(())
}

fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file_content = String::new();
    let mut file = fs::File::open(path)?;
    file.read_to_string(&mut file_content)?;

    Ok(file_content)
}

fn construct_document(content: &str) {
    let d: Document = quick_xml::de::from_str(content).unwrap();
    println!("{:?}", d);
}
