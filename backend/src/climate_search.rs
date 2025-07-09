use backend::{get_relative_minmax, minmax, reduce_minmax};
use common::cities::City;
use rayon::{prelude::*, result};

#[derive(Debug)]
pub struct ClimateSearchItem<'a> {
    id: usize,
    city: &'a City,
    coords: GeoCoordinates,
    relative_minmax: ClimateMinMax,
}

#[derive(Clone, Copy, Debug)]
pub struct GeoCoordinates {
    /// Latitude in radians
    phi: f64,
    /// Longitude in radians
    lambda: f64,
}

pub fn make_climate_search_items<'a>(cities: &'a Vec<City>) -> Vec<ClimateSearchItem<'a>> {
    let start = std::time::Instant::now();

    let total_minmax = cities.par_iter()
        .map(|city| get_climate_min_max(city))
        .reduce_with(|a, b| ClimateMinMax {
            humidity: reduce_minmax(a.humidity, b.humidity),
            ppt: reduce_minmax(a.ppt, b.ppt),
            srad: reduce_minmax(a.srad, b.srad),
            tmax: reduce_minmax(a.tmax, b.tmax),
            tmin: reduce_minmax(a.tmin, b.tmin),
            ws: reduce_minmax(a.ws, b.ws),
        })
        .unwrap();

    let items = cities.par_iter().enumerate()
        .map(|(index, city)| {
            let coords = GeoCoordinates {
                phi: city.latitude.to_radians(),
                lambda: city.longitude.to_radians(),
            };
            let climate_minmax = get_climate_min_max(city);
            let relative_minmax = ClimateMinMax {
                humidity: get_relative_minmax(climate_minmax.humidity, total_minmax.humidity),
                ppt: get_relative_minmax(climate_minmax.ppt, total_minmax.ppt),
                srad: get_relative_minmax(climate_minmax.srad, total_minmax.srad),
                tmax: get_relative_minmax(climate_minmax.tmax, total_minmax.tmax),
                tmin: get_relative_minmax(climate_minmax.tmin, total_minmax.tmin),
                ws: get_relative_minmax(climate_minmax.ws, total_minmax.ws),
            };
            ClimateSearchItem {
                id: index,
                city,
                coords,
                relative_minmax,
            }
        })
        .collect();

    // items.sort_by_key(|item| item.id);
    eprintln!("Built climate search items in {} ms", start.elapsed().as_millis());

    items
}

#[derive(Debug)]
pub struct ClimateMinMax {
    humidity: (f32, f32),
    ppt: (f32, f32),
    srad: (f32, f32),
    tmax: (f32, f32),
    tmin: (f32, f32),
    ws: (f32, f32),
}

fn get_climate_min_max(city: &City) -> ClimateMinMax {
    let humidity_u32 = minmax(&city.climate.humidity_monthly);
    let humidity = (humidity_u32.0 as f32, humidity_u32.1 as f32);
    let ppt = minmax(&city.climate.ppt_monthly);
    let srad = minmax(&city.climate.srad_monthly);
    let tmax = minmax(&city.climate.tmax_monthly);
    let tmin = minmax(&city.climate.tmin_monthly);
    let ws = minmax(&city.climate.ws_monthly);
    ClimateMinMax {
        humidity,
        ppt,
        srad,
        tmax,
        tmin,
        ws,
    }
}

#[derive(Debug)]
pub struct ClimateScoredItem<'a> {
    id: usize,
    city: &'a City,
    coords: GeoCoordinates,
    diff: f32,
    distance_km: f64,
}

pub fn search_climate<'a>(items: &'a Vec<ClimateSearchItem>, query: &'a ClimateSearchItem) -> Vec<ClimateScoredItem<'a>> {
    let min_dist = 200.0;

    let mut scored_items = items.par_iter().enumerate()
        .map(|(index, item)| ClimateScoredItem {
            id: index,
            city: item.city,
            coords: item.coords,
            diff: get_climate_diff(item, query),
            distance_km: get_distance_km(&item.coords, &query.coords)
        })
        .filter(|item|
            item.id == query.id || item.diff < 1.5 && item.distance_km >= min_dist
        )
        .collect::<Vec<_>>();

    // eprintln!("Found {} candidates", scored_items.len());

    scored_items.par_sort_by(|a, b| a.diff.total_cmp(&b.diff));

    let mut result_items = Vec::<ClimateScoredItem>::new();
    for item in scored_items {
        if result_items.iter().all(|existing_res_it|
            get_distance_km(&item.coords, &existing_res_it.coords) >= min_dist
        ) {
            result_items.push(item);
        }
    }

    // eprintln!("Found {} results", result_items.len());

    result_items.into_iter().take(10).collect()
}

fn get_climate_diff(item: &ClimateSearchItem, query: &ClimateSearchItem) -> f32 {
    let a = &item.relative_minmax;
    let b = &query.relative_minmax;
    diff_minmax(a.humidity, b.humidity)
        + diff_minmax(a.ppt, b.ppt)
        + diff_minmax(a.srad, b.srad)
        + diff_minmax(a.tmax, b.tmax)
        + diff_minmax(a.tmin, b.tmin)
        + diff_minmax(a.ws, b.ws)
}

fn diff_minmax(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

const EARTH_RADIUS_KM: f64 = 6371.0;

/// https://en.wikipedia.org/wiki/Great-circle_distance#Formulae
fn get_distance_km(a: &GeoCoordinates, b: &GeoCoordinates) -> f64 {
    EARTH_RADIUS_KM * (a.phi.sin() * b.phi.sin() + a.phi.cos() * b.phi.cos() * (a.lambda - b.lambda).abs().cos()).acos()

    // approx distance
    // let dx = EARTH_RADIUS_KM * (a.lambda - b.lambda).abs() * ((a.phi + b.phi) / 2.0).cos();
    // let dy = EARTH_RADIUS_KM * (a.phi - b.phi).abs();
    // (dx * dx + dy * dy).sqrt()
}
