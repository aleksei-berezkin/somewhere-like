use backend::library::{earth::{arc_length_to_chord_length, get_arc_distance_km, get_cartesian_distance_km_squared, get_cartesian_xyz}, minmax::{diff_minmax, diff_minmax_maybe, get_relative_minmax, minmax, minmax_maybe, reduce_minmax, reduce_minmax_maybe}};
use common::{city::City, util::round_0_1_and_assert_finite};
use rayon::prelude::*;

use crate::api::{ClimateSearchResult, ClimateSearchResultItem};

pub struct ClimateSearchData<'a> {
    /// Order: same as in the Cities list
    items: Vec<ClimateSearchItem<'a>>,
}

#[derive(Debug)]
struct ClimateSearchItem<'a> {
    /// Index in the Cities list
    id: usize,
    city: &'a City,
    cartesian_xyz: [f64; 3],
    relative_minmax: ClimateMinMax,
}

#[derive(Debug)]
struct ClimateMinMax {
    humidity: Option<(f32, f32)>,
    ppt: (f32, f32),
    srad: (f32, f32),
    tmax: (f32, f32),
    tmin: (f32, f32),
    ws: (f32, f32),
}

pub fn make_climate_search_data<'a>(cities: &'a Vec<City>) -> ClimateSearchData<'a> {
    let start = std::time::Instant::now();

    let total_minmax = cities.par_iter()
        .map(|city| get_climate_min_max(city))
        .reduce_with(|a, b| ClimateMinMax {
            humidity: reduce_minmax_maybe(a.humidity, b.humidity),
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
                humidity: climate_minmax.humidity.map(|h| get_relative_minmax(h, total_minmax.humidity.unwrap())),
                ppt: get_relative_minmax(climate_minmax.ppt, total_minmax.ppt),
                srad: get_relative_minmax(climate_minmax.srad, total_minmax.srad),
                tmax: get_relative_minmax(climate_minmax.tmax, total_minmax.tmax),
                tmin: get_relative_minmax(climate_minmax.tmin, total_minmax.tmin),
                ws: get_relative_minmax(climate_minmax.ws, total_minmax.ws),
            };
            ClimateSearchItem {
                id: index,
                city,
                cartesian_xyz: get_cartesian_xyz(city.latitude, city.longitude),
                relative_minmax,
            }
        })
        .collect();

    eprintln!("Built climate search items in {} ms", start.elapsed().as_millis());

    ClimateSearchData { items }
}

fn get_climate_min_max(city: &City) -> ClimateMinMax {
    let humidity = minmax_maybe(&city.climate.humidity_monthly);
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
struct ClimateScoredItem<'a> {
    id: usize,
    city: &'a City,
    cartesian_xyz: &'a [f64; 3],
    diff: f32,
}

pub fn search_climate<'a>(data: &'a ClimateSearchData, id: usize) -> ClimateSearchResult<'a> {
    let started = std::time::Instant::now();
    let query_maybe = &data.items.get(id);
    if query_maybe.is_none() {
        return ClimateSearchResult {
            items: vec![],
            elapsed_ms: started.elapsed().as_millis() as u32,
        };
    }
    let query = query_maybe.unwrap();

    // Chord length is much faster to calculate, using it for filtering
    let min_chord_length = arc_length_to_chord_length(200.0);
    let min_chord_length_sq = min_chord_length * min_chord_length;

    let (scored_items, max_diff) = score_items(&data.items, query);

    let mut filtered_items = Vec::<ClimateScoredItem>::new();

    filtered_items.push(ClimateScoredItem {
        id: query.id,
        city: query.city,
        cartesian_xyz: &query.cartesian_xyz,
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

    let result_items = filtered_items.into_iter()
        .map(|item| ClimateSearchResultItem {
            id: item.id,
            city: item.city,
            distance_km: round_0_1_and_assert_finite(
                get_arc_distance_km(
                    item.city.latitude,
                    item.city.longitude,
                    query.city.latitude,
                    query.city.longitude,
                )
            ),
            similarity_percent: 100.0 * (1.0 - item.diff / max_diff),
        })
        .take(20)
        .collect::<Vec<_>>();

    ClimateSearchResult {
        items: result_items,
        elapsed_ms: started.elapsed().as_millis() as u32,
    }
}

fn score_items<'a>(items: &'a Vec<ClimateSearchItem>, query: &'a ClimateSearchItem) -> (Vec<ClimateScoredItem<'a>>, f32) {
    let scored_items = items.par_iter().enumerate()
        .map(|(index, item)| ClimateScoredItem {
            id: index,
            city: item.city,
            cartesian_xyz: &item.cartesian_xyz,
            diff: get_climate_diff(item, query),
        })
        .collect::<Vec<_>>();

    let max_diff = scored_items.par_iter()
        .max_by(|a, b| a.diff.total_cmp(&b.diff))
        .unwrap().diff;

    let mut scored_above_half_sorted = scored_items.into_par_iter()
        .filter(|item| item.diff < max_diff / 2.0)
        .collect::<Vec<_>>();
    
    scored_above_half_sorted.par_sort_by(|a, b| a.diff.total_cmp(&b.diff));

    (scored_above_half_sorted, max_diff)
}

fn get_climate_diff(item: &ClimateSearchItem, query: &ClimateSearchItem) -> f32 {
    let a = &item.relative_minmax;
    let b = &query.relative_minmax;
    diff_minmax_maybe(a.humidity, b.humidity).unwrap_or(0.0)
        + diff_minmax(a.ppt, b.ppt)
        + diff_minmax(a.srad, b.srad)
        + diff_minmax(a.tmax, b.tmax)
        + diff_minmax(a.tmin, b.tmin)
        + diff_minmax(a.ws, b.ws)
}
