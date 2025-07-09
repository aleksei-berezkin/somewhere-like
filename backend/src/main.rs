mod climate_search;
mod search;
use common::utils::eprintln_memory_usage;
use search::{make_search_data, search_cities};

use crate::{climate_search::{make_climate_search_items, search_climate}, search::make_search_query};

fn main() {
    let cities = common::cities::read_cities();
    let search_data = make_search_data(&cities);
    let climate_search_items = make_climate_search_items(&cities);

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

        let id_maybe: Result<u32, _> = query.parse();
        if let Ok(id) = id_maybe {
            let found_items = search_climate(&climate_search_items, &climate_search_items[id as usize]);
            found_items.iter().for_each(|f| eprintln!("{:?}", f));
        } else {
            let city_search_query = make_search_query(query);
    
            let found_items = search_cities(&search_data, &city_search_query);
            found_items.iter().for_each(|f| eprintln!("{:?}", f));
    
        }

        eprintln!("Done \"{}\" in {} ms", query, started.elapsed().as_millis());
    }
}
