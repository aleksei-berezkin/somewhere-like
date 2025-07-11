mod climate_search;
mod search;
mod api;
use common::utils::eprintln_memory_usage;
use search::{make_search_data, search_cities};

use crate::{climate_search::{make_climate_search_data, search_climate}, search::make_search_query};

fn main() {
    let cities = common::cities::read_cities();
    let search_data = make_search_data(&cities);
    let climate_search_data = make_climate_search_data(&cities);

    eprintln_memory_usage();
    eprintln!("Enter city name to search by name, or id to search by climate");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        let query = buf.trim();
        if query.is_empty() {
            continue;
        }

        let started = std::time::Instant::now();

        let id_maybe: Result<usize, _> = query.parse();
        if let Ok(id) = id_maybe {
            let climate_search_result = search_climate(&climate_search_data, id);
            climate_search_result.items.into_iter().for_each(|f| eprintln!("{:?}", f));
        } else {
            let city_search_query = make_search_query(query);
            let city_search_result = search_cities(&search_data, &city_search_query);
            city_search_result.items.into_iter().for_each(|f| eprintln!("{:?}", f));
        }

        eprintln!("Done \"{}\" in {} ms", query, started.elapsed().as_millis());
    }
}
