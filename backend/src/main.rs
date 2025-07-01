mod search;
use common::utils::eprintln_memory_usage;
use search::{make_search_data, search_cities};

use crate::search::make_search_query;

fn main() {
    let cities = common::cities::read_cities();
    let search_data = make_search_data(&cities);
    eprintln_memory_usage();
    eprintln!("Enter city name");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        let query = buf.trim();
        if query.is_empty() {
            continue;
        }

        let city_search_query = make_search_query(query);

        let started = std::time::Instant::now();
        let found_items = search_cities(&search_data, &city_search_query);
        found_items.iter().for_each(|f| eprintln!("{:?}", f));

        eprintln!("Done \"{}\" in {} ms, found.len(): {}", query, started.elapsed().as_millis(), found_items.len());
    }
}
