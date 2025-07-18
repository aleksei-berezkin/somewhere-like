use crate::library::{api::*, climate_search::*, search::*};

use common::{city::City, city_csv::read_cities};
use once_cell::sync::Lazy;


pub fn handle_request(req_str: String, simple_allowed: bool) -> Result<String, String> {
    let request = parse_request(&req_str, simple_allowed);
    if let Err(msg) = request {
        return Err(msg)
    }
    let result = handle_request_impl(request.unwrap());
    Ok(serde_json::to_string(&result).unwrap())
}

fn parse_request(req_str: &str, simple_allowed: bool) -> Result<CityRequest, String> {
    if let Ok(req_json) = serde_json::from_str::<CityRequest>(req_str) {
        return Ok(req_json)
    }

    if !simple_allowed || req_str.contains("{") || req_str.contains("}") {
        return Err(format!("Invalid request: {}", req_str))
    }

    let id_maybe: Result<usize, _> = req_str.parse();
    let simple_req =
        if let Ok(id) = id_maybe {
            CityRequest::SearchClimate(ClimateSearchRequest {
                city_id: id,
                start_index: None,
                max_items: None,
            })
        } else {
            CityRequest::SearchCity(CitySearchRequest {
                query: req_str.into(),
                start_index: None,
                max_items: None,
            })
        };

    return Ok(simple_req)
}


fn handle_request_impl<'a>(request: CityRequest) -> CityResult<'a> {
    match request {
        CityRequest::SearchCity(req) => {
            let cities = &CACHED_DATA.cities;
            let search_data = &CACHED_DATA.search_data;
            let city_search_query = make_search_query(&req.query);
            let search_result = search_cities(
                cities,
                search_data,
                &city_search_query,
                req.start_index.unwrap_or(SEARCH_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(SEARCH_DEFAULT_MAX_ITEMS),
            );
            CityResult::SearchCity(search_result)
        },
        CityRequest::SearchClimate(req) => {
            let cities = &CACHED_DATA.cities;
            let climate_search_data = &CACHED_DATA.climate_search_data;
            let climate_search_result = search_climate(
                cities,
                climate_search_data,
                req.city_id,
                req.start_index.unwrap_or(CLIMATE_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(CLIMATE_DEFAULT_MAX_ITEMS),
            );
            CityResult::SearchClimate(climate_search_result)
        },
    }
}

static CACHED_DATA: Lazy<CachedData> = Lazy::new(|| {
    let cities = read_cities();
    let search_data = make_search_data(&cities);
    let climate_search_data = make_climate_search_data(&cities);
    let data = CachedData { cities, search_data, climate_search_data };
    data
});

struct CachedData {
    cities: Vec<City>,
    search_data: CitySearchData,
    climate_search_data: ClimateSearchData,
}
