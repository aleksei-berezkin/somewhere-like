use common::city::City;
use crate::library::{api::*, intern::*, jaro::jaro_winkler_vec, split::split_name_rest};
use rayon::prelude::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use thread_local::ThreadLocal;


pub struct CitySearchData {
    search_items: Vec<CitySearchItem>,
    intern_registry: InternRegistry,
}

#[derive(Debug)]
struct CitySearchItem {
    /// Simply index in the cities list
    id: usize,
    names_lowercase: Vec<InternId>,
    admin_unit_lowercase: Option<InternId>,
    country_lowercase: InternId,
}

pub fn make_search_data(cities: &Vec<City>) -> CitySearchData {
    let start = std::time::Instant::now();
    let intern_builder = InternBuilder::new();
    let intern_lowercase = |s: &str| {
        intern_builder.intern(s.to_lowercase().chars().collect())
    };

    let search_items = cities.par_iter().enumerate()
        .map(|(index, city)| {
            let names_lowercase = city.names.iter()
                .map(|it| intern_lowercase(it))
                .collect::<Vec<_>>();
            let admin_unit_lowercase = city.admin_unit.as_ref()
                .map(|it| intern_lowercase(it));
            let country_lowercase = intern_lowercase(&city.country);
            CitySearchItem {
                id: index,
                names_lowercase,
                admin_unit_lowercase,
                country_lowercase,
            }
        })
        .collect();
    eprintln!("Built search items in {} ms", start.elapsed().as_millis());

    CitySearchData {
        intern_registry: intern_builder.build(),
        search_items
    }
}

pub struct CitySearchQuery {
    name_rest_variants: Vec<(InternId, Option<InternId>)>,
    intern_registry: InternRegistry,
    cache: ThreadLocal<RefCell<Vec<f32>>>,
    // Counting hits and misses has some overhead.
    // Should not be made in production.
    cache_hit_miss_count: (AtomicUsize, AtomicUsize),
}

pub fn make_search_query(query: &str) -> CitySearchQuery {
    let lowercase_query = query.trim().to_lowercase();
    let intern_builder = InternBuilder::new();
    let name_rest_variants = split_name_rest(&lowercase_query).iter()
        .map(|(name, rest)| (
            intern_builder.intern(name.chars().collect()),
            rest.map(|r| intern_builder.intern(r.chars().collect()))
        ))
        .collect();
    CitySearchQuery {
        name_rest_variants,
        intern_registry: intern_builder.build(),
        cache: ThreadLocal::new(),
        cache_hit_miss_count: (AtomicUsize::new(0), AtomicUsize::new(0)),
    }
}


pub fn search_cities<'a>(cities: &'a Vec<City>, search_data: &'a CitySearchData, search_query: &CitySearchQuery, start_index: usize, max_items: usize) -> CitySearchResponse<'a> {
    let started = std::time::Instant::now();
    let mut items = search_data.search_items
        .par_iter()
        .map(
            |item| {
                score_city(&cities[item.id], item, &search_data.intern_registry, search_query, &search_query.cache, &search_query.cache_hit_miss_count)
            }
        )
        .filter(|item| item.score > 0.85)
        .collect::<Vec<_>>();

    let hit = search_query.cache_hit_miss_count.0.load(Ordering::Relaxed);
    let miss = search_query.cache_hit_miss_count.1.load(Ordering::Relaxed);

    items.sort_by(|a, b| b.score.total_cmp(&a.score));
    CitySearchResponse {
        items: items.into_iter().skip(start_index).take(max_items).collect::<Vec<_>>(),
        elapsed_ms: started.elapsed().as_millis() as u32,
        cache_hit_rate_percent: 100.0 * (hit as f32 / (hit + miss) as f32),
    }
}

const NAME_POSITION_WEIGHT: f32 = -0.001;
const POPULATION_LOG_WEIGHT: f32 = 0.01;
const ADMIN_UNIT_WEIGHT: f32 = 0.25;
const COUNTRY_WEIGHT: f32 = 0.25;

fn score_city<'a>(
    city: &'a City,
    search_item: &'a CitySearchItem,
    city_intern_registry: &InternRegistry,
    city_search_query: &CitySearchQuery,
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
) -> CitySearchResponseItem<'a> {
    city_search_query.name_rest_variants.iter()
        .flat_map(|query_name_and_rest| {
            search_item.names_lowercase.iter().enumerate()
                .map(|city_name_index_and_name| {
                    let score = score_city_impl(
                        city_name_index_and_name,
                        &search_item.admin_unit_lowercase,
                        &search_item.country_lowercase,
                        city.population,
                        query_name_and_rest,
                        cache,
                        cache_hit_miss_count,
                        city_intern_registry,
                        &city_search_query.intern_registry
                    );
                    CitySearchResponseItem {
                        id: search_item.id,
                        score,
                        matched_name: &city.names[city_name_index_and_name.0],
                        name: &city.names[0],
                        population: city.population,
                        admin_unit: &city.admin_unit,
                        country: &city.country
                    }
                })
        })
        .max_by(|a, b| a.score.total_cmp(&b.score))
        .unwrap()
}

fn score_city_impl(
    city_name_index_and_name: (usize, &InternId),
    city_admin_unit_maybe: &Option<InternId>,
    city_country: &InternId,
    city_population: u64,
    query_name_and_rest: &(InternId, Option<InternId>),
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
    city_intern_registry: &InternRegistry,
    query_intern_registry: &InternRegistry,
) -> f32 {
    let (city_name_index, city_name) = city_name_index_and_name;
    let (query_name, query_rest_maybe) = query_name_and_rest;

    let name_similarity = jaro_winkler_cached(city_name, query_name, cache, cache_hit_miss_count, city_intern_registry, query_intern_registry);
    let (
        admin_unit_similarity,
        country_similarity
    ) = if let Some(query_rest) = query_rest_maybe {
        (
            jaro_winkler_cached(city_country, query_rest, cache, cache_hit_miss_count, city_intern_registry, query_intern_registry),
            if let Some(city_admin_unit) = city_admin_unit_maybe {
                jaro_winkler_cached(city_admin_unit, query_rest, cache, cache_hit_miss_count, city_intern_registry, query_intern_registry)
            } else {
                0.0
            },
        )
    } else {
        (
            0.0,
            0.0,
        )
    };

    name_similarity
        + NAME_POSITION_WEIGHT * city_name_index as f32
        + POPULATION_LOG_WEIGHT * (city_population as f32).log10()
        + ADMIN_UNIT_WEIGHT * admin_unit_similarity
        + COUNTRY_WEIGHT * country_similarity
}

fn jaro_winkler_cached(
    city_str: &InternId,
    query_str: &InternId,
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
    city_intern_registry: &InternRegistry,
    query_intern_registry: &InternRegistry
) -> f32 {
    let mut cache = cache
        .get_or(|| RefCell::new(vec![-1.0_f32; city_intern_registry.len() as usize * query_intern_registry.len() as usize]))
        .borrow_mut();

    let index = (*city_str * query_intern_registry.len() + *query_str) as usize;
    let cached_score = cache[index];
    if cached_score >= 0.0 {
        cache_hit_miss_count.0.fetch_add(1, Ordering::Relaxed);
        return cached_score;
    }

    cache_hit_miss_count.1.fetch_add(1, Ordering::Relaxed);

    let score = jaro_winkler_vec(
        city_intern_registry.resolve(*city_str).unwrap(),
        query_intern_registry.resolve(*query_str).unwrap()
    );
    cache[index] = score;
    score
}
