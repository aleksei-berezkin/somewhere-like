use common::cities::City;
use backend::{intern::{InternedId, Interner, InternerBuilder}, jaro_winkler_vec, split_name_rest};
use rayon::prelude::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use thread_local::ThreadLocal;

pub struct CitySearchData<'a> {
    pub search_items: Vec<CitySearchItem<'a>>,
    pub interner: Interner,
}

#[derive(Debug)]
pub struct CitySearchItem<'a> {
    /// Simply index in the cities list
    id: usize,
    city: &'a City,
    names_lowercase: Vec<InternedId>,
    admin_unit_lowercase: Option<InternedId>,
    country_lowercase: InternedId,
}

pub fn make_search_data(cities: &Vec<City>) -> CitySearchData {
    let start = std::time::Instant::now();
    let interner_builder = InternerBuilder::new();
    let intern_lowercase = |s: &str| {
        interner_builder.intern(s.to_lowercase().chars().collect())
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
                city,
                names_lowercase,
                admin_unit_lowercase,
                country_lowercase,
            }
        })
        .collect();
    eprintln!("Built search items in {} ms", start.elapsed().as_millis());

    CitySearchData {
        interner: interner_builder.into_interner(),
        search_items
    }
}

pub struct CitySearchQuery {
    name_rest_variants: Vec<(InternedId, Option<InternedId>)>,
    interner: Interner,
    cache: ThreadLocal<RefCell<Vec<f32>>>,
    // Counting hits and misses has some overhead.
    // Should not be made in production.
    cache_hit_miss_count: (AtomicUsize, AtomicUsize),
}

pub fn make_search_query(query: &str) -> CitySearchQuery {
    let lowercase_query = query.trim().to_lowercase();
    let interner_builder = InternerBuilder::new();
    let name_rest_variants = split_name_rest(&lowercase_query).iter()
        .map(|(name, rest)| (
            interner_builder.intern(name.chars().collect()),
            rest.map(|r| interner_builder.intern(r.chars().collect()))
        ))
        .collect();
    CitySearchQuery {
        name_rest_variants,
        interner: interner_builder.into_interner(),
        cache: ThreadLocal::new(),
        cache_hit_miss_count: (AtomicUsize::new(0), AtomicUsize::new(0)),
    }
}

#[derive(Debug)]
pub struct CityScoredItem<'a> {
    id: usize,
    score: f32,
    matched_name: &'a str,
    name: &'a str,
    population: u64,
    admin_unit: &'a Option<String>,
    country: &'a str,
}

pub fn search_cities<'a>(search_data: &'a CitySearchData<'a>, search_query: &CitySearchQuery) -> Vec<CityScoredItem<'a>> {
    let mut found_items = search_data.search_items
        .par_iter()
        .map(
            |item| {
                score_city(item, &search_data.interner, search_query, &search_query.cache, &search_query.cache_hit_miss_count)
            }
        )
        .filter(|item| item.score > 0.85)
        .collect::<Vec<_>>();

    let hit = search_query.cache_hit_miss_count.0.load(Ordering::Relaxed);
    let miss = search_query.cache_hit_miss_count.1.load(Ordering::Relaxed);

    eprintln!(
        "jaro_winkler hit {}, miss {}, hit rate {:.2}",
        hit, miss, 100.0 *(hit as f64 / (hit + miss) as f64)
    );

    found_items.sort_by(|a, b| b.score.total_cmp(&a.score));
    found_items.into_iter().take(10).collect()
}

const NAME_POSITION_WEIGHT: f32 = -0.001;
const POPULATION_LOG_WEIGHT: f32 = 0.01;
const ADMIN_UNIT_WEIGHT: f32 = 0.25;
const COUNTRY_WEIGHT: f32 = 0.25;

fn score_city<'a>(
    search_item: &'a CitySearchItem,
    city_interner: &Interner,
    city_search_query: &CitySearchQuery,
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
) -> CityScoredItem<'a> {
    city_search_query.name_rest_variants.iter()
        .flat_map(|query_name_and_rest| {
            search_item.names_lowercase.iter().enumerate()
                .map(|city_name_index_and_name| {
                    let score = score_city_impl(
                        city_name_index_and_name,
                        &search_item.admin_unit_lowercase,
                        &search_item.country_lowercase,
                        search_item.city.population,
                        query_name_and_rest,
                        cache,
                        cache_hit_miss_count,
                        city_interner,
                        &city_search_query.interner
                    );
                    CityScoredItem {
                        id: search_item.id,
                        score,
                        matched_name: &search_item.city.names[city_name_index_and_name.0],
                        name: &search_item.city.names[0],
                        population: search_item.city.population,
                        admin_unit: &search_item.city.admin_unit,
                        country: &search_item.city.country
                    }
                })
        })
        .max_by(|a, b| a.score.total_cmp(&b.score))
        .unwrap()
}

fn score_city_impl(
    city_name_index_and_name: (usize, &InternedId),
    city_admin_unit_maybe: &Option<InternedId>,
    city_country: &InternedId,
    city_population: u64,
    query_name_and_rest: &(InternedId, Option<InternedId>),
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
    city_interner: &Interner,
    query_interner: &Interner,
) -> f32 {
    let (city_name_index, city_name) = city_name_index_and_name;
    let (query_name, query_rest_maybe) = query_name_and_rest;

    let name_similarity = jaro_winkler_cached(city_name, query_name, cache, cache_hit_miss_count, city_interner, query_interner);
    let (
        admin_unit_similarity,
        country_similarity
    ) = if let Some(query_rest) = query_rest_maybe {
        (
            jaro_winkler_cached(city_country, query_rest, cache, cache_hit_miss_count, city_interner, query_interner),
            if let Some(city_admin_unit) = city_admin_unit_maybe {
                jaro_winkler_cached(city_admin_unit, query_rest, cache, cache_hit_miss_count, city_interner, query_interner)
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
    city_str: &InternedId,
    query_str: &InternedId,
    cache: &ThreadLocal<RefCell<Vec<f32>>>,
    cache_hit_miss_count: &(AtomicUsize, AtomicUsize),
    city_interner: &Interner,
    query_interner: &Interner
) -> f32 {
    let mut cache = cache
        .get_or(|| RefCell::new(vec![-1.0_f32; city_interner.len() as usize * query_interner.len() as usize]))
        .borrow_mut();

    let index = (*city_str * query_interner.len() + *query_str) as usize;
    let cached_score = cache[index];
    if cached_score >= 0.0 {
        cache_hit_miss_count.0.fetch_add(1, Ordering::Relaxed);
        return cached_score;
    }

    cache_hit_miss_count.1.fetch_add(1, Ordering::Relaxed);

    let score = jaro_winkler_vec(
        city_interner.resolve(*city_str).unwrap(),
        query_interner.resolve(*query_str).unwrap()
    );
    cache[index] = score;
    score
}
