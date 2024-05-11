use std::io::{self, Read};
use std::error::Error;
use std::fs;
use serde::Deserialize;
use crate::common::{Depth, Seconds};
use crate::stats;

#[derive(Deserialize)]
pub struct UDDFDoc {
    #[serde(rename = "profiledata")]
    pub profile_data: ProfileDataElem,
    #[serde(rename = "gasdefinitions")]
    pub gas_definitions: GasDefinition,
}

#[derive(Debug, Deserialize)]
pub struct GasDefinition {
    #[serde(rename = "mix")]
    pub gas_mixes: Option<Vec<Mix>>,
}

#[derive(Debug, Deserialize)]
pub struct Mix {
    #[serde(rename = "@id")]
    pub id: String,
    pub name: String,
    pub o2: f64,
    pub n2: Option<f64>,
    pub he: Option<f64>,
}

#[derive(Deserialize)]
pub struct ProfileDataElem {
    #[serde(rename = "repetitiongroup")]
    pub repetition_group: Vec<RepetitionGroupElem>,
}

#[derive(Deserialize)]
pub struct RepetitionGroupElem {
    #[serde(rename = "dive")]
    pub dives: Vec<DiveElem>,
}

#[derive(Debug, Deserialize)]
pub struct DiveElem {
    pub samples: SampleElem,
    #[serde(rename = "informationbeforedive")]
    pub information_before_dive: InfoElem,
}

#[derive(Debug, Deserialize)]
pub struct InformationBeforeDiveElem {
    #[serde(rename = "informationbeforedive")]
    pub information_before_dive: InfoElem,
}

#[derive(Debug, Deserialize)]
pub struct InfoElem {
    #[serde(rename = "surfacepressure")]
    pub surface_pressure: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct SampleElem {
    #[serde(rename = "waypoint")]
    pub waypoints: Vec<WaypointElem>,
}

#[derive(Debug, Deserialize)]
pub struct WaypointElem {
    #[serde(rename = "divetime")]
    pub dive_time: Seconds,
    pub depth: Depth,
    pub switchmix: Option<SwitchMix>,
    #[serde(rename = "decostop")]
    pub decostops: Option<Vec<DecostopElem>>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchMix {
    #[serde(rename="@ref")]
    pub gas_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct DecostopElem {
    #[serde(rename = "@kind")]
    pub kind: String,
}

pub fn parse_file(file_path: &str) -> Result<UDDFDoc, Box<dyn Error>> {
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
