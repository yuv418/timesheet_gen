use cairo::{Context, ImageSurface};
use poppler::Document;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::PathBuf,
};
use strfmt::{strfmt, Format};

mod timesheet_generator;
mod timesheet_info;

use crate::{
    timesheet_generator::generate_timesheet,
    timesheet_info::{TimesheetData, TimesheetInfo},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    // TODO clean up args parsing
    let ts_info: TimesheetInfo = match args.get(2) {
        Some(ts_i_path) => {
            let ts_path = File::open(fs::canonicalize(ts_i_path)?)?;
            serde_json::from_reader(ts_path)?
        }
        None => {
            println!("Usage: timesheet_gen [timesheet] [info_json]");
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid parameters")));
        }
    };

    match args.get(1) {
        Some(s) => {
            match fs::canonicalize(s)?.to_str() {
                Some(timesheet_abspath) => {
                    // Load the document into Poppler
                    println!("Timesheet path is {}", timesheet_abspath);

                    // Try to get the page from Poppler and convert it into an image
                    generate_timesheet(PathBuf::from(timesheet_abspath), ts_info)
                }
                None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Could not convert timesheet path to string"))),
            }
        }
        None => {
            println!("Usage: timesheet_gen [timesheet] [info_json]");
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid parameters")))
        }
    }
}
