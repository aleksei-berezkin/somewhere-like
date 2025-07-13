use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub type City = BaseCity<Vec<String>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseCity<NamesT> {
    pub names: NamesT,
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
