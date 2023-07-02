use cairo::{Context, ImageSurface};
use chrono::NaiveDate;
use poppler::Document;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    ops::Deref,
};
use strfmt::{strfmt, Format};

mod timesheet_info;

use timesheet_info::Timesheet;

const DATE_FORMAT: &str = "%m/%d/%Y";

use crate::timesheet_info::TimesheetData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    // TODO clean up args parsing
    let ts_info: Timesheet = match args.get(2) {
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
                    let ts = Document::from_file(&format!("file://{}", timesheet_abspath), None)?;

                    // Try to get the page from Poppler and convert it into an image
                    match ts.page(0) {
                        Some(first_page) => {
                            // Create Cairo context

                            // NOTE we might have to adjust the DPI and things like that before calling size()
                            let (w, h) = first_page.size();
                            println!("Width {} height {}", w, h);

                            let surface = ImageSurface::create(cairo::Format::ARgb32, w as i32, h as i32)?;

                            let context = Context::new(&surface)?;

                            first_page.render(&context);

                            // Note to self: when drawing a rectangle, make sure to call fill (or stroke) afterwards
                            // Set colour to red (we will probably change this later)
                            context.set_source_rgb(1.0, 0.0, 0.0);

                            // Draw some sample text
                            context.set_font_size(15.0);
                            context.select_font_face("Liberation Serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);

                            // Draw positional data onto PDF
                            for pos_entry in ts_info.pos_data {
                                context.set_font_size(pos_entry.font_size);
                                context.move_to(pos_entry.pos.0, pos_entry.pos.1);
                                match pos_entry.data_value {
                                    TimesheetData::Date => {
                                        context.show_text(&chrono::offset::Local::now().date_naive().format(DATE_FORMAT).to_string())?;
                                    }
                                    TimesheetData::Str(s) => {
                                        context.show_text(&s)?;
                                    }
                                };
                            }

                            // Draw entry box
                            // 1: Calculate size of text line

                            let entry_font_size = {
                                // We add 1 because the last row contains total information for hours and wage
                                let entry_num = ts_info.entries.len() as f64 + 1.0;
                                let box_height = ts_info.entry_pos_data.box_bottom.1 - ts_info.entry_pos_data.box_top.1;
                                let padding_total_height = entry_num * ts_info.entry_pos_data.row_padding;

                                let text_line_height = (box_height - padding_total_height) / entry_num;

                                // The max font size cannot be used, so we have to make it even smaller to accomodate all the entries in the box
                                if text_line_height < ts_info.entry_pos_data.max_font_size {
                                    text_line_height
                                } else {
                                    ts_info.entry_pos_data.max_font_size
                                }
                            };

                            context.set_font_size(entry_font_size);

                            // 2: Draw each entry onto the pdf

                            let mut row_y = ts_info.entry_pos_data.box_top.1;
                            let mut max_line_height = 0.0;

                            let mut total_hours = 0.0;
                            let mut total_amnt = 0.0;
                            let mut total_map = HashMap::new();

                            for entry in ts_info.entries {
                                for (k, v) in &ts_info.entry_format {
                                    let v = v.to_string();
                                    context.move_to(
                                        *ts_info.entry_pos_data.entry_starts.get(k).expect("Missing column start for timesheet entry"),
                                        row_y,
                                    );
                                    let mut text = "".to_string();

                                    // Special cases
                                    if k == "total" {
                                        let hours =
                                            entry.get("hours").expect("Expected rate field").as_f64().expect("Invalid hours format");
                                        let rate = entry.get("rate").expect("Expected rate field").as_f64().expect("Invalid rate format");
                                        let amnt = hours * rate;

                                        let mut map = HashMap::new();
                                        map.insert("total".to_string(), amnt);

                                        text = strfmt(&v, &map)?;

                                        total_hours += hours;
                                        total_amnt += amnt;

                                        // Lazy way to do this
                                        total_map.insert("hours".to_string(), total_hours);
                                        total_map.insert("total".to_string(), total_amnt);
                                    } else {
                                        // BUG: This breaks all number formatting
                                        let entry_strings: HashMap<String, String> = entry
                                            .iter()
                                            .map(|(k, v)| {
                                                (
                                                    k.to_owned(),
                                                    match v.as_str() {
                                                        Some(s) => s.to_owned(),
                                                        None => v
                                                            .as_f64()
                                                            .expect("Expected either string or f64 entry, but neither was found")
                                                            .to_string()
                                                            .to_owned(),
                                                    },
                                                )
                                            })
                                            .collect();
                                        text = v.format(&entry_strings)?;
                                    }

                                    context.show_text(&text)?;
                                    let line_height = context.text_extents(&text)?.height();
                                    if line_height > max_line_height {
                                        max_line_height = line_height;
                                    };
                                }
                                row_y += max_line_height + ts_info.entry_pos_data.row_padding;
                            }

                            // Hour/total information in last row (TODO should this be hardcoded?)

                            context.move_to(
                                *ts_info
                                    .entry_pos_data
                                    .entry_starts
                                    .get("hours")
                                    .expect("Missing column start for total hours in timesheet entry"),
                                row_y,
                            );

                            context.show_text(&ts_info.entry_format.get("hours").unwrap().format(&total_map)?)?;

                            context.move_to(
                                *ts_info
                                    .entry_pos_data
                                    .entry_starts
                                    .get("total")
                                    .expect("Missing column start for total amount in timesheet entry"),
                                row_y,
                            );
                            context.show_text(&ts_info.entry_format.get("total").unwrap().format(&total_map)?)?;

                            println!("Finished drawing information onto pdf");

                            // PNG currently for debugging purposes
                            let mut output_f = File::create("output.png")?;
                            Ok(surface.write_to_png(&mut output_f)?)
                        }
                        None => Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Timesheet does not have at least one page",
                        ))),
                    }
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
