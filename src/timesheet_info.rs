use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
// We need to insert the date when we generate the timesheet if the option exists

#[derive(Serialize, Deserialize)]
pub struct TimesheetEntries {
    pub date: NaiveDate,
    pub description: String,
    pub rate: f32,
}

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
    pub entries: Vec<TimesheetEntries>,
}
