use backend::library::minmax::{get_relative_minmax, minmax, reduce_minmax};
use common::cities::City;
use rayon::prelude::*;

#[derive(Debug)]
pub struct ClimateSearchItem<'a> {
    id: usize,
    city: &'a City,
    relative_minmax: ClimateMinMax,
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
                relative_minmax,
            }
        })
        .collect();

    eprintln!("Built climate search items in {} ms", start.elapsed().as_millis());

    items
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
    /// Direct (chorde) distance is much faster than the precise arc distance
    cartesian_xyz: [f64; 3],
    diff: f32,
}

#[derive(Debug)]
pub struct ClimateResultItem<'a> {
    id: usize,
    city: &'a City,
    distance_km: f64,
    similarity_percent: f32,
}

pub fn search_climate<'a>(items: &'a Vec<ClimateSearchItem>, query: &'a ClimateSearchItem) -> Vec<ClimateResultItem<'a>> {
    let min_chord_length = arc_length_to_chord_length(200.0);
    let min_chord_length_sq = min_chord_length * min_chord_length;

    let (scored_items, max_diff) = score_items(items, query);

    eprintln!("Max diff {}", max_diff);

    let mut filtered_items = Vec::<ClimateScoredItem>::new();

    filtered_items.push(ClimateScoredItem {
        id: query.id,
        city: query.city,
        cartesian_xyz: get_cartesian_xyz(query.city),
        diff: 0.0,
    });

    for item in scored_items {
        // It's faster than spatial index (e.g. rstar)
        if filtered_items.iter().all(|existing_res_it|
            get_cartesian_distance_km_squared(&item.cartesian_xyz, &existing_res_it.cartesian_xyz) >= min_chord_length_sq
        ) {
            filtered_items.push(item);
        }
    }

    eprintln!("Found {} results", filtered_items.len());

    filtered_items.into_iter()
        .map(|item| ClimateResultItem {
            id: item.id,
            city: item.city,
            distance_km: get_arc_distance_km(&item.city, &query.city),
            similarity_percent: 100.0 * (1.0 - item.diff / max_diff),
        })
        .take(20)
        .collect()
}

fn score_items<'a>(items: &'a Vec<ClimateSearchItem>, query: &'a ClimateSearchItem) -> (Vec<ClimateScoredItem<'a>>, f32) {
    let scored_items = items.par_iter().enumerate()
        .map(|(index, item)| ClimateScoredItem {
            id: index,
            city: item.city,
            cartesian_xyz: get_cartesian_xyz(item.city),
            diff: get_climate_diff(item, query),
        })
        .collect::<Vec<_>>();

    let max_diff = scored_items.par_iter()
        .max_by(|a, b| a.diff.total_cmp(&b.diff))
        .unwrap().diff;

    let mut scored_sorted = scored_items.into_par_iter()
        .filter(|item| item.diff < max_diff / 2.0)
        .collect::<Vec<_>>();
    
    scored_sorted.par_sort_by(|a, b| a.diff.total_cmp(&b.diff));

    (scored_sorted, max_diff)
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

/// https://en.wikipedia.org/wiki/Chord_(geometry)#In_trigonometry
fn arc_length_to_chord_length(a: f64) -> f64 {
    let theta = a / EARTH_RADIUS_KM;
    EARTH_RADIUS_KM * 2.0 * (theta / 2.0).sin()
}

fn get_cartesian_xyz(city: &City) -> [f64; 3] {
    [
        EARTH_RADIUS_KM * city.latitude.to_radians().cos() * city.longitude.to_radians().cos(),
        EARTH_RADIUS_KM * city.latitude.to_radians().cos() * city.longitude.to_radians().sin(),
        EARTH_RADIUS_KM * city.latitude.to_radians().sin(),
    ]
}

fn get_cartesian_distance_km_squared(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// https://en.wikipedia.org/wiki/Great-circle_distance#Formulae
fn get_arc_distance_km(a: &City, b: &City) -> f64 {
    let phi_a = a.latitude.to_radians();
    let phi_b = b.latitude.to_radians();
    let lambda_a = a.longitude.to_radians();
    let lambda_b = b.longitude.to_radians();
    EARTH_RADIUS_KM * (phi_a.sin() * phi_b.sin() + phi_a.cos() * phi_b.cos() * (lambda_a - lambda_b).abs().cos()).acos()
}
