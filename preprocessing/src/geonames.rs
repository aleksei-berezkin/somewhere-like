use std::{collections::HashMap, fmt::Debug, fs::File, io::{BufRead, BufReader}, str::FromStr};
use chrono::NaiveDate;
use common::{cities::{assert_city_name}, utils::get_data_in_dir};

#[derive(Debug)]
pub struct GeonamesCity {
    pub names: Vec<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub admin_code: String,
    pub population: u64,
    pub elevation: Option<i32>,
    pub region: String,
    pub modification_date: NaiveDate,
}

pub fn read_geonames_cities() -> Vec<GeonamesCity> {
    let reader = BufReader::new(File::open(get_data_in_dir().join("cities5000.txt")).unwrap());
    reader.lines().enumerate()
        .map(|(index, line_res)|
            parse_geonames_line(line_res.unwrap(), index + 1)
        )
        .collect()
}

fn parse_geonames_line(line: String, line_number: usize) -> GeonamesCity {
    let fields: Vec<&str> = line.split('\t').collect();

    let names: Vec<String> = std::iter::once(fields[1])
        .chain(std::iter::once(fields[2]))
        .chain(fields[3].split(',').filter(|s| !s.is_empty()))
        .map(str::to_owned)
        .collect();

    names.iter().for_each(|n| assert_city_name(n));

    let latitude_str = fields[4];
    let latitude = f64::from_str(latitude_str).expect(&format!("Invalid latitude: \"{}\" on line {}", latitude_str, line_number));
    let longitude_str = fields[5];
    let longitude = f64::from_str(longitude_str).expect(&format!("Invalid latitude: \"{}\" on line {}", longitude_str, line_number));
    let country_code = fields[8].to_owned();
    let admin_code = fields[10].to_owned();
    let population_str = fields[14];
    let population = u64::from_str(population_str).expect(&format!("Invalid population: \"{}\" on line {}", population_str, line_number));
    let elevation_str = fields[15];
    let elevation = match elevation_str {
        "" => None,
        _ => Some(i32::from_str(elevation_str).expect(&format!("Invalid elevation: \"{}\" on line {}", elevation_str, line_number))),
    };
    let dem_str = fields[16];
    let dem = match dem_str {
        "" => None,
        _ => Some(i32::from_str(dem_str).expect(&format!("Invalid dem: \"{}\" on line {}", dem_str, line_number))),
    };
    let region = fields[17].to_owned().split("/").next().unwrap().to_owned();
    let modification_date_str = fields[18];
    let modification_date = NaiveDate::parse_from_str(modification_date_str, "%Y-%m-%d").expect(&format!("Invalid modification date: \"{}\" on line {}", modification_date_str, line_number));

    GeonamesCity {
        names,
        latitude,
        longitude,
        country_code,
        admin_code,
        population,
        elevation: dem.or(elevation),
        region,
        modification_date,
    }
}

pub fn read_geonames_country_names() -> HashMap<String, String> {
    return read_key_value_file("countryInfo.txt", 0, 4);
}

pub fn read_admin_codes() -> HashMap<String, String> {
    return read_key_value_file("admin1CodesASCII.txt", 0, 1);
}

fn read_key_value_file(file_name: &str, key_index: usize, value_index: usize) -> HashMap<String, String> {
    let reader = BufReader::new(File::open(get_data_in_dir().join(file_name)).unwrap());
    reader.lines()
        .filter_map(|line_result| {
            let line = line_result.unwrap();
            if line.is_empty() || line.starts_with('#') {
                None
            } else {
                let items = line.split('\t').collect::<Vec<_>>();
                Some((items[key_index].to_owned(), items[value_index].to_owned()))
            }
        })
        .collect()
}
