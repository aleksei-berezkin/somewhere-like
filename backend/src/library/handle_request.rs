use crate::library::{api::*, climate_search::*, search::*};

use common::{city::City, city_csv::read_cities};
use once_cell::sync::Lazy;


pub fn handle_request(req_str: String, is_cli: bool) -> Result<String, String> {
    parse_request(&req_str, is_cli)
        .and_then(|request| {
            let response = handle_request_impl(request);
            Ok(to_string_response(response, is_cli))
        })
}

fn parse_request(req_str: &str, is_cli: bool) -> Result<CityRequest, String> {
    let simple_cmd_allowed = is_cli && !req_str.contains("{") && !req_str.contains("}");
    let req_json_res = serde_json::from_str::<CityRequest>(req_str).map_err(|e| e.to_string());
    if req_json_res.is_ok() || !simple_cmd_allowed {
        return req_json_res
    }

    let simple_req =
        if let Ok(id) = req_str.parse() {
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

    Ok(simple_req)
}


fn handle_request_impl<'a>(request: CityRequest) -> CityResponse<'a> {
    match request {
        CityRequest::SearchCity(req) => {
            let cities = &CACHED_DATA.cities;
            let search_data = &CACHED_DATA.search_data;
            let city_search_query = make_search_query(&req.query);
            let search_response = search_cities(
                cities,
                search_data,
                &city_search_query,
                req.start_index.unwrap_or(SEARCH_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(SEARCH_DEFAULT_MAX_ITEMS),
            );
            CityResponse::SearchCity(search_response)
        },
        CityRequest::SearchClimate(req) => {
            let cities = &CACHED_DATA.cities;
            let climate_search_data = &CACHED_DATA.climate_search_data;
            let climate_search_response = search_climate(
                cities,
                climate_search_data,
                req.city_id,
                req.start_index.unwrap_or(CLIMATE_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(CLIMATE_DEFAULT_MAX_ITEMS),
            );
            CityResponse::SearchClimate(climate_search_response)
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

macro_rules! to_str_items {
    ($items: expr) => {
        $items.into_iter()
            .map(|it| serde_json::to_string(&it).unwrap())
            .collect::<Vec<String>>()
            .join("\n")
    };
}

fn to_string_response<'a>(response: CityResponse<'a>, is_cli: bool) -> String {
    if is_cli {
        match response {
            CityResponse::SearchCity(search_response) =>{
                let items_str = to_str_items!(search_response.items);
                format!("{}\ncache_hit_rate_percent: {}",
                    items_str,
                    search_response.cache_hit_rate_percent,
                )
            },
            CityResponse::SearchClimate(climate_search_response) =>
                to_str_items!(climate_search_response.items),
        }
    } else {
        serde_json::to_string(&response).unwrap()
    }
}
