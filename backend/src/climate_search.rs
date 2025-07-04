use backend::minmax;
use common::cities::City;
use rayon::prelude::*;

#[derive(Debug)]
pub struct ClimateSearchItem<'a> {
    id: usize,
    city: &'a City,
    humidity_minmax: (u8, u8),
    ppt_minmax: (f32, f32),
    srad_minmax: (f32, f32),
    tmax_minmax: (f32, f32),
    tmin_minmax: (f32, f32),
    ws_minmax: (f32, f32),
}

pub fn make_climate_search_data<'a>(cities: &'a Vec<City>) -> Vec<ClimateSearchItem<'a>> {
    let start = std::time::Instant::now();
    let items = cities.par_iter().enumerate()
        .map(|(index, city)| get_climate_search_item(index, city))
        .collect();

    eprintln!("Built climate search items in {} ms", start.elapsed().as_millis());
    items
}

fn get_climate_search_item<'a>(id: usize, city: &'a City) -> ClimateSearchItem<'a> {
    let humidity_minmax = minmax(&city.climate.humidity_monthly);
    let ppt_minmax = minmax(&city.climate.ppt_monthly);
    let srad_minmax = minmax(&city.climate.srad_monthly);
    let tmax_minmax = minmax(&city.climate.tmax_monthly);
    let tmin_minmax = minmax(&city.climate.tmin_monthly);
    let ws_minmax = minmax(&city.climate.ws_monthly);
    ClimateSearchItem {
        id,
        city,
        humidity_minmax,
        ppt_minmax,
        srad_minmax,
        tmax_minmax,
        tmin_minmax,
        ws_minmax,
    }
}
