mod climate_search;
mod search;
mod api;
use std::{io::Write, process::ExitCode};

use common::{city_csv::read_cities, util::eprintln_memory_usage};
use search::{make_search_data, search_cities};

use crate::{api::{CityCommand, CityResult, CitySearchRequest, ClimateSearchRequest, CLIMATE_DEFAULT_MAX_ITEMS, CLIMATE_DEFAULT_START_INDEX, SEARCH_DEFAULT_MAX_ITEMS, SEARCH_DEFAULT_START_INDEX}, climate_search::{make_climate_search_data, search_climate, ClimateSearchData}, search::{make_search_query, CitySearchData}};


macro_rules! eprintln_json_items {
    ($items: expr) => {
        $items.into_iter().for_each(|f| {
            serde_json::to_writer(std::io::stderr(), &f).unwrap();
            eprintln!();
        })
    };
}

fn main() -> ExitCode {
    let cities = read_cities();
    let search_data = make_search_data(&cities);
    let climate_search_data = make_climate_search_data(&cities);

    eprintln_memory_usage();

    let cmd_arg = std::env::args().nth(1);
    if let Some(cmd_arg_str) = cmd_arg {
        let command = parse_json_command(&cmd_arg_str);
        if command.is_none() {
            eprintln!("Unknown command: {}", cmd_arg_str);
            return ExitCode::FAILURE;
        }
        let exec_res = execute_command(command.unwrap(), &search_data, &climate_search_data);
        serde_json::to_writer(std::io::stdout(), &exec_res).unwrap();
        std::io::stdout().write(b"\n").unwrap();
        std::io::stdout().flush().unwrap();
        return ExitCode::SUCCESS
    }

    eprintln!("Enter city name to search by name, or id to search by climate; or use json messages");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        let command_str = buf.trim();
        if command_str.is_empty() {
            continue;
        }

        let command = parse_json_or_simple_command(command_str);
        if command.is_none() {
            eprintln!("Unknown command: {}", command_str);
            continue;
        }

        let started = std::time::Instant::now();

        let result = execute_command(command.unwrap(), &search_data, &climate_search_data);
        match result {
            CityResult::SearchCity(res) => eprintln_json_items!(res.items),
            CityResult::SearchClimate(res) => eprintln_json_items!(res.items),
        }

        eprintln!("Done \"{}\" in {} ms", command_str, started.elapsed().as_millis());
    }
}

fn parse_json_command(command_str: &str) -> Option<CityCommand> {
    serde_json::from_str::<CityCommand>(command_str).ok()
}

fn parse_json_or_simple_command(command_str: &str) -> Option<CityCommand> {
    if command_str.starts_with('{') {
        return parse_json_command(command_str);
    }

    let id_maybe: Result<usize, _> = command_str.parse();
    let simple_cmd =
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

    return Some(simple_cmd)
}

fn execute_command<'a>(command: CityCommand, search_data: &'a CitySearchData, climate_search_data: &'a ClimateSearchData) -> CityResult<'a> {
    match command {
        CityCommand::SearchCity(req) => {
            let city_search_query = make_search_query(&req.query);
            let search_result = search_cities(
                &search_data,
                &city_search_query,
                req.start_index.unwrap_or(SEARCH_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(SEARCH_DEFAULT_MAX_ITEMS),
            );
            CityResult::SearchCity(search_result)
        },
        CityCommand::SearchClimate(req) => {
            let climate_search_result = search_climate(
                &climate_search_data,
                req.city_id,
                req.start_index.unwrap_or(CLIMATE_DEFAULT_START_INDEX),
                req.max_items.unwrap_or(CLIMATE_DEFAULT_MAX_ITEMS),
            );
            CityResult::SearchClimate(climate_search_result)
        },
    }
}
