use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// We need to insert the date when we generate the timesheet if the option exists

// UNUSED
#[derive(Serialize, Deserialize)]
pub struct TimesheetEntries {
    pub date: NaiveDate,
    pub hours: f32,
    pub description: String,
    pub rate: f32,
}

// Include column offsets, box top/bottom etc. (TODO make it work for entries that do not go by row, but by column)
// Include row spacing, as some timesheets may have some kind of row divider
// We can use the box top/bottom to calculate the font size
// TODO allow for multiple boxes
#[derive(Serialize, Deserialize)]
pub struct TimesheetEntryPositionalData {
    pub box_top: (f64, f64),
    pub box_bottom: (f64, f64),

    // Date/description may want to be split for flexibility later
    pub entry_starts: HashMap<String, f64>,

    // Distance between each row
    pub row_padding: f64,

    // Maximum font size. If the required font size is less than the max font size, then the font size will be reduced.
    #[serde(default = "max_font_size")]
    pub max_font_size: f64,
}

fn max_font_size() -> f64 { 12.0 }

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum TimesheetData {
    Date,
    Str(String),
}

// TODO colour and font?
#[derive(Serialize, Deserialize)]
pub struct TimesheetPositionalData {
    pub data_name: String, // Eg. "name" or "address"
    pub data_value: TimesheetData,
    pub pos: (f64, f64),
    pub font_size: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Timesheet {
    pub pos_data: Vec<TimesheetPositionalData>,
    pub entries: Vec<HashMap<String, Value>>,

    // First argument is the parameter to format!, and all resulting are keys in the entries
    pub entry_format: HashMap<String, String>,

    pub entry_pos_data: TimesheetEntryPositionalData,
}
