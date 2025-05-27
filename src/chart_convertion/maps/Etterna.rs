#![allow(warnings)] // REMOVE LATER WHEN FINALLY ADDING SUPPORT

use super::chart_classes;
use super::errors;

pub(crate) fn parse_ett(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>>  {
    unimplemented!();
}

pub(crate) fn convert_to_ett(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
    unimplemented!();
}