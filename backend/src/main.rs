mod climate_search;
mod search;
mod api;
use std::io::Write;

use common::{city_csv::read_cities, util::eprintln_memory_usage};
use search::{make_search_data, search_cities};

use crate::{api::{CityCommand, CitySearchRequest, ClimateSearchRequest, CLIMATE_DEFAULT_MAX_ITEMS, CLIMATE_DEFAULT_START_INDEX, SEARCH_DEFAULT_MAX_ITEMS, SEARCH_DEFAULT_START_INDEX}, climate_search::{make_climate_search_data, search_climate}, search::make_search_query};

macro_rules! eprintln_json_items {
    ($items: expr) => {
        $items.into_iter().for_each(|f| {
            serde_json::to_writer(std::io::stderr(), &f).unwrap();
            eprintln!();
        })
    };
}

fn main() {
    let cities = read_cities();
    let search_data = make_search_data(&cities);
    let climate_search_data = make_climate_search_data(&cities);

    eprintln_memory_usage();
    eprintln!("Enter city name to search by name, or id to search by climate; or use json messages");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        let command_str = buf.trim();
        if command_str.is_empty() {
            continue;
        }

        let command = parse_command(command_str);

        let started = std::time::Instant::now();
        match command.command {
            CityCommand::SearchCity(req) => {
                let city_search_query = make_search_query(&req.query);
                let city_search_result = search_cities(
                    &search_data,
                    &city_search_query,
                    req.start_index.unwrap_or(SEARCH_DEFAULT_START_INDEX),
                    req.max_items.unwrap_or(SEARCH_DEFAULT_MAX_ITEMS),
                );
                if command.is_json {
                    serde_json::to_writer(std::io::stdout(), &city_search_result).unwrap()
                } else {
                    eprintln_json_items!(city_search_result.items)
                }
            },
            CityCommand::SearchClimate(req) => {
                let climate_search_result = search_climate(
                    &climate_search_data,
                    req.city_id,
                    req.start_index.unwrap_or(CLIMATE_DEFAULT_START_INDEX),
                    req.max_items.unwrap_or(CLIMATE_DEFAULT_MAX_ITEMS),
                );
                if command.is_json {
                    serde_json::to_writer(std::io::stdout(), &climate_search_result).unwrap()
                } else {
                    eprintln_json_items!(climate_search_result.items)
                }
            },
        }

        if command.is_json {
            std::io::stdout().write(b"\n").unwrap();
            std::io::stdout().flush().unwrap();
        } else {
            eprintln!("Done \"{}\" in {} ms", command_str, started.elapsed().as_millis());
        }
    }
}


struct ParsedCommand {
    command: CityCommand,
    is_json: bool,
}

fn parse_command(command_str: &str) -> ParsedCommand {
    let json_command_res = serde_json::from_str::<CityCommand>(command_str);
    if let Ok(command) = json_command_res {
        return ParsedCommand {
            command,
            is_json: true,
        };
    }

    let id_maybe: Result<usize, _> = command_str.parse();
    let command =
        if let Ok(id) = id_maybe {
            CityCommand::SearchClimate(ClimateSearchRequest {
                city_id: id,
                start_index: None,
                max_items: None,
            })
        } else {
            CityCommand::SearchCity(CitySearchRequest {
                query: command_str.into(),
                start_index: None,
                max_items: None,
            })
        };

        return ParsedCommand {
        command,
        is_json: false
    }
}
