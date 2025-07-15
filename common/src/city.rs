use chrono::NaiveDate;
use derive_csv_friendly::CsvFriendly;
use serde::{Deserialize, Serialize};

#[derive(Debug, CsvFriendly, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct City {
    pub names: Vec<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub admin_unit: Option<String>,
    pub country: String,
    pub population: u64,
    pub elevation: Option<i32>,
    pub region: String,
    pub modification_date: NaiveDate,
    pub climate: CityClimate,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityClimate {
    pub humidity_monthly: [Option<f32>; 12],
    pub ppt_monthly: [f32; 12],
    pub srad_monthly: [f32; 12],
    pub tmax_monthly: [f32; 12],
    pub tmin_monthly: [f32; 12],
    pub ws_monthly: [f32; 12],
}
