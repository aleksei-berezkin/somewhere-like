use std::{fs::File, io::BufRead, iter::once};
use rayon::prelude::*;
use crate::{city::{BaseCity, City}, util::get_data_out_dir};

type CityCsv = BaseCity<String>;

const NAMES_DELIMITER: &str = "|";

fn city_to_csv(city: City) -> CityCsv {
    city.names.iter().for_each(|name|
        assert!(!name.contains(NAMES_DELIMITER), "The name contains a delimiter: {}", name)
    );
    CityCsv {
        names: city.names.join(NAMES_DELIMITER),
        latitude: city.latitude,
        longitude: city.longitude,
        admin_unit: city.admin_unit,
        country: city.country,
        population: city.population,
        elevation: city.elevation,
        region: city.region,
        modification_date: city.modification_date,
        climate: city.climate
    }
}

fn city_from_csv(city: CityCsv) -> City {
    City {
        names: city.names.split(NAMES_DELIMITER).map(&str::to_owned).collect(),
        latitude: city.latitude,
        longitude: city.longitude,
        admin_unit: city.admin_unit,
        country: city.country,
        population: city.population,
        elevation: city.elevation,
        region: city.region,
        modification_date: city.modification_date,
        climate: city.climate
    }
}

const FILE_NAME: &str = "cities.csv";

pub fn write_cities(cities: Vec<City>) {
    let path = get_data_out_dir().join(FILE_NAME);
    eprintln!("Writing to {:?}", path);
    let file = File::create(path).unwrap();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_writer(&file);
    cities.into_iter().for_each(
        |c| writer.serialize(&city_to_csv(c)).unwrap()
    );
    writer.flush().unwrap();
    let file_size = file.metadata().unwrap().len();
    eprintln!("Written {:.1} MB", (file_size as f64) / 1024.0 / 1024.0);
}

pub fn read_cities() -> Vec<City> {
    let started = std::time::Instant::now();

    let path = get_data_out_dir().join(FILE_NAME);
    let lines = std::io::BufReader::new(File::open(&path).unwrap())
        .lines()
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    let cities = lines.par_chunks(2000)
        .flat_map(|chunk| {
            // String is slightly faster here than Vec<u8>
            let chunk_str = chunk.iter()
                .flat_map(|line| once(line.as_str()).chain(once("\n")))
                .collect::<String>();
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b'\t')
                .from_reader(chunk_str.as_bytes());
            reader.deserialize::<CityCsv>()
                .map(|res| city_from_csv(res.unwrap()))
                .collect::<Vec<City>>()
        })
        .collect::<Vec<_>>();
    eprintln!("From {:?} loaded {} cities in {} ms", path, cities.len(), started.elapsed().as_millis());
    cities
}
