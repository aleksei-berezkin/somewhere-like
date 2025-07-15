use common::city::City;
use serde::{Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchResult<'a> {
    pub items: Vec<CitySearchResultItem<'a>>,
    pub elapsed_ms: u32,
    pub cache_hit_rate_percent: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CitySearchResultItem<'a> {
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
pub struct ClimateSearchResult<'a> {
    pub items: Vec<ClimateSearchResultItem<'a>>,
    pub elapsed_ms: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClimateSearchResultItem<'a> {
    pub id: usize,
    pub city: &'a City,
    pub distance_km: f64,
    pub similarity_percent: f32,
}
