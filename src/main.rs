use cairo::{Context, ImageSurface};
use chrono::NaiveDate;
use poppler::Document;
use std::{
    env,
    fs::{self, File},
    ops::Deref,
};

mod timesheet_info;

use timesheet_info::Timesheet;

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
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid parameters",
            )));
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

                            let surface =
                                ImageSurface::create(cairo::Format::ARgb32, w as i32, h as i32)?;

                            let context = Context::new(&surface)?;

                            first_page.render(&context);

                            // Note to self: when drawing a rectangle, make sure to call fill (or stroke) afterwards
                            // Set colour to red (we will probably change this later)
                            context.set_source_rgb(1.0, 0.0, 0.0);

                            // Draw some sample text
                            context.set_font_size(15.0);
                            context.select_font_face(
                                "Liberation Serif",
                                cairo::FontSlant::Normal,
                                cairo::FontWeight::Bold,
                            );

                            // Draw positional data onto PDF
                            for pos_entry in ts_info.pos_data {
                                context.move_to(pos_entry.pos.0, pos_entry.pos.1);
                                match pos_entry.data_value {
                                    TimesheetData::Date => {
                                        context.show_text(
                                            &chrono::offset::Local::now()
                                                .date_naive()
                                                .format("%m/%d/%Y")
                                                .to_string(),
                                        )?;
                                    }
                                    TimesheetData::Str(s) => {
                                        context.show_text(&s)?;
                                    }
                                };
                            }

                            println!("Finished drawing rectangle onto pdf");

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
                None => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Could not convert timesheet path to string",
                ))),
            }
        }
        None => {
            println!("Usage: timesheet_gen [timesheet] [info_json]");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid parameters",
            )))
        }
    }
}
