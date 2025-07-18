use common::city::City;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "camelCase")]
pub enum CityRequest {
    SearchCity(CitySearchRequest),
    SearchClimate(ClimateSearchRequest),
}

/// Example:
/// `{"command": "searchCity", "query": "Tokyo", "startIndex": 0, "maxItems": 4}`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchRequest {
    pub query: String,
    pub start_index: Option<usize>,
    pub max_items: Option<usize>,
}

pub const SEARCH_DEFAULT_START_INDEX: usize = 0;
pub const SEARCH_DEFAULT_MAX_ITEMS: usize = 10;

/// Example:
/// `{"command": "searchClimate", "cityId": 34040, "startIndex": 0, "maxItems": 5}`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClimateSearchRequest {
    pub city_id: usize,
    pub start_index: Option<usize>,
    pub max_items: Option<usize>,
}

pub const CLIMATE_DEFAULT_START_INDEX: usize = 0;
pub const CLIMATE_DEFAULT_MAX_ITEMS: usize = 100;

#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "camelCase")]
pub enum CityResponse<'a> {
    SearchCity(CitySearchResponse<'a>),
    SearchClimate(ClimateSearchResponse<'a>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchResponse<'a> {
    pub items: Vec<CitySearchResponseItem<'a>>,
    pub elapsed_ms: u32,
    pub cache_hit_rate_percent: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchResponseItem<'a> {
    pub id: usize,
    pub score: f32,
    pub matched_name: &'a str,
    pub name: &'a str,
    pub population: u64,
    pub admin_unit: &'a Option<String>,
    pub country: &'a str,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClimateSearchResponse<'a> {
    pub items: Vec<ClimateSearchResponseItem<'a>>,
    pub elapsed_ms: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClimateSearchResponseItem<'a> {
    pub id: usize,
    pub city: &'a City,
    pub distance_km: f64,
    pub similarity_percent: f32,
}
