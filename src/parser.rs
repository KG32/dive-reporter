use std::io::{self, Read};
use std::error::Error;
use std::fs;
use serde::Deserialize;
use crate::stats;

#[derive(Deserialize)]
pub struct UDDFDoc {
    #[serde(rename = "profiledata")]
    pub profile_data: ProfileDataElem,
}

#[derive(Deserialize)]
pub struct ProfileDataElem {
    #[serde(rename = "repetitiongroup")]
    pub repetition_group: RepetitionGroupElem,
}

#[derive(Deserialize)]
pub struct RepetitionGroupElem {
    pub dive: DiveElem,
}

#[derive(Deserialize)]
pub struct DiveElem {
    pub samples: SampleElem,
}

#[derive(Deserialize)]
pub struct SampleElem {
    #[serde(rename = "waypoint")]
    pub waypoints: Vec<WaypointElem>,
}

#[derive(Debug, Deserialize)]
pub struct WaypointElem {
    #[serde(rename = "divetime")]
    pub dive_time: stats::Seconds,
    pub depth: stats::Depth,
    #[serde(rename = "decostop")]
    pub decostops: Option<Vec<DecostopElem>>,
}

#[derive(Debug, Deserialize)]
pub struct DecostopElem {
    #[serde(rename = "@kind")]
    pub kind: String,
}

pub fn parse_file(file_path: &str) -> Result<UDDFDoc, Box<dyn Error>> {
    println!("Parsing file {}", file_path);
    let file_content = read_file_content(file_path)?;
    let document = construct_from_uddf(&file_content)?;

    Ok(document)
}

fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file_content = String::new();
    let mut file = fs::File::open(path)?;
    file.read_to_string(&mut file_content)?;

    Ok(file_content)
}

fn construct_from_uddf(content: &str) -> Result<UDDFDoc, Box<dyn Error>> {
    let document: UDDFDoc = quick_xml::de::from_str(content)?;
    Ok(document)
}
