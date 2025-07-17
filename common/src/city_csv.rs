use core::panic;
use std::fs::File;
use rayon::prelude::*;
use crate::{city::{CityCsvFriendly, City}, util::get_data_out_dir};

const SHARD_SIZE: usize = 2_000;
const EXPECTED_SHARDS_NUM: usize = 32;

pub fn get_file_name(shard: usize) -> String {
    let max_shard_str_len = (EXPECTED_SHARDS_NUM - 1).to_string().len();
    format!("cities-{:0fill$}.csv", shard, fill = max_shard_str_len)
}

pub fn write_cities(cities: Vec<City>) {
    let actual_shards_num = (cities.len() as f64 / SHARD_SIZE as f64).ceil() as usize;
    if actual_shards_num != EXPECTED_SHARDS_NUM {
        panic!("Expected {} shards, got {}", EXPECTED_SHARDS_NUM, actual_shards_num);
    }

    let csv_cities = cities.into_par_iter()
        .map(CityCsvFriendly::from)
        .collect::<Vec<_>>();

    let total_size: u64 = csv_cities.par_chunks(SHARD_SIZE).enumerate()
        .map(|(shard, chunk)| {
            let path = get_data_out_dir().join(get_file_name(shard));
            let file = File::create(path).unwrap();
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .delimiter(b'\t')
                .from_writer(&file);
            chunk.into_iter().for_each(
                |c| writer.serialize(c).unwrap()
            );
            writer.flush().unwrap();
            file.metadata().unwrap().len()
        })
        .sum();

    eprintln!("Written totally {:.1} MB to {} shards", (total_size as f64) / 1024.0 / 1024.0, actual_shards_num);
}

pub fn read_cities() -> Vec<City> {
    let started = std::time::Instant::now();

    let cities = (0..EXPECTED_SHARDS_NUM).into_par_iter()
        .flat_map(|shard| {
            let path = get_data_out_dir().join(get_file_name(shard));
            let file_rd = std::io::BufReader::new(File::open(&path).unwrap());

            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b'\t')
                .from_reader(file_rd);
            reader.deserialize::<CityCsvFriendly>()
                .map(|res| City::from(res.unwrap()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    eprintln!("From {:?} shards loaded {} cities in {} ms", EXPECTED_SHARDS_NUM, cities.len(), started.elapsed().as_millis());
    cities
}
