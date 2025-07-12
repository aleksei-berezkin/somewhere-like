use std::{fs::File, io::BufRead, iter::once};

use chrono::NaiveDate;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{cities::serde_vec::VEC_DELIMITER, utils::get_data_out_dir};

pub type City = BaseCity;

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseCity {
    #[serde(with = "serde_vec")]
    pub names: Vec<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub admin_unit: Option<String>,
    pub country: String,
    pub population: u64,
    pub elevation: Option<i32>,
    pub region: String,
    pub modification_date: NaiveDate,
    pub climate: CityClimate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CityClimate {
    pub humidity_monthly: [Option<f32>; 12],
    pub ppt_monthly: [f32; 12],
    pub srad_monthly: [f32; 12],
    pub tmax_monthly: [f32; 12],
    pub tmin_monthly: [f32; 12],
    pub ws_monthly: [f32; 12],
}

pub fn assert_city_name(name: &str) {
    if name.contains(VEC_DELIMITER) {
        panic!("Invalid name: {}", name);
    }
}

const FILE_NAME: &str = "cities.csv";

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
            reader.deserialize().map(Result::unwrap).collect::<Vec<City>>()
        })
        .collect::<Vec<_>>();
    eprintln!("From {:?} loaded {} cities in {} ms", path, cities.len(), started.elapsed().as_millis());
    cities
}

pub fn write_cities(cities: &Vec<City>) {
    let path = get_data_out_dir().join(FILE_NAME);
    eprintln!("Writing to {:?}", path);
    let file = File::create(path).unwrap();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_writer(&file);
    cities.iter().for_each(|city| writer.serialize(&city).unwrap());
    writer.flush().unwrap();
    let file_size = file.metadata().unwrap().len();
    eprintln!("Written {:.1} MB", (file_size as f64) / 1024.0 / 1024.0);
}

pub mod serde_vec {
    use serde::{Deserializer, Serializer, de::Error};
    use super::*;

    pub const VEC_DELIMITER: &str = "|";

    pub trait ToCsvString: ToString {
        fn to_csv_string(&self) -> String {
            self.to_string()
        }
    }

    impl ToCsvString for String {}
    impl ToCsvString for u8 {}
    impl ToCsvString for f32 {
        fn to_csv_string(&self) -> String {
            if self.fract() == 0.0 {
                format!("{}", *self as i32)
            } else {
                format!("{:.1}", self)
            }
        }
    }

    pub fn serialize<S: Serializer, T: ToCsvString>(vec: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error> {
        let s = vec.iter().map(ToCsvString::to_csv_string).collect::<Vec<_>>().join(VEC_DELIMITER);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: std::str::FromStr>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T::Err: std::fmt::Display,
    {
        (String::deserialize(deserializer)?)
            .split(VEC_DELIMITER)
            .map(str::parse::<T>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(D::Error::custom)
    }
}
