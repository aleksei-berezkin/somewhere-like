use common::{cities::{write_cities, City, CityClimate}, utils::eprintln_memory_usage};

mod terra_climate;
use rayon::prelude::*;
use terra_climate::TerraClimateData;

mod geonames;
use geonames::{read_admin_codes, read_geonames_cities, read_geonames_country_names, GeonamesCity};

fn main() {
    let started = std::time::Instant::now();

    let geonames_cities = read_geonames_cities();
    let country_code_to_name = read_geonames_country_names();
    let admin_code_to_name = read_admin_codes();
    let all_terra_climate = get_all_terra_climate();

    let cities: Vec<City> = geonames_cities.into_par_iter()
        .filter_map(|geo_city| {
            get_city_climate(&geo_city, &all_terra_climate)
                .map(|climate|
                    City {
                        names: geo_city.names,
                        latitude: geo_city.latitude,
                        longitude: geo_city.longitude,
                        admin_unit: admin_code_to_name.get(&(geo_city.country_code.clone() + "." + &geo_city.admin_code)).map(String::to_owned),
                        country: country_code_to_name.get(&geo_city.country_code).expect(&format!("Country code \"{}\" not found", geo_city.country_code)).to_owned(),
                        population: geo_city.population,
                        elevation: geo_city.elevation,
                        region: geo_city.region,
                        modification_date: geo_city.modification_date,
                        climate: climate
                    }
                )
        })
        .collect();

    eprintln_memory_usage();
    write_cities(&cities);
    eprintln!("Done {} cities in {:.2} sec", cities.len(), started.elapsed().as_secs_f32());
}

struct AllTerraClimate {
    ppt: TerraClimateData<i32>,
    srad: TerraClimateData<i16>,
    tmin: TerraClimateData<i16>,
    tmax: TerraClimateData<i16>,
    vap: TerraClimateData<i16>,
    vpd: TerraClimateData<i16>,
    ws: TerraClimateData<i16>,
}

fn get_all_terra_climate() -> AllTerraClimate {
    let ppt = TerraClimateData::<i32>::new("ppt");
    let srad = TerraClimateData::<i16>::new("srad");
    let tmax = TerraClimateData::<i16>::new("tmax");
    let tmin = TerraClimateData::<i16>::new("tmin");
    let vap = TerraClimateData::<i16>::new("vap");
    let vpd = TerraClimateData::<i16>::new("vpd");
    let ws = TerraClimateData::<i16>::new("ws");
    return AllTerraClimate {ppt, srad, tmax, tmin, vap, vpd, ws};
}

fn get_city_climate(city: &GeonamesCity, all_terra_climate: &AllTerraClimate) -> Option<CityClimate> {
    let lat = city.latitude;
    let lon = city.longitude;
    let name = &city.names[0];

    let ppt_monthly = all_terra_climate.ppt.get_monthly_values(lat, lon, name)?;
    let srad_monthly = all_terra_climate.srad.get_monthly_values(lat, lon, name)?;
    let tmax_monthly = all_terra_climate.tmax.get_monthly_values(lat, lon, name)?;
    let tmin_monthly = all_terra_climate.tmin.get_monthly_values(lat, lon, name)?;
    let vap_monthly = all_terra_climate.vap.get_monthly_values(lat, lon, name)?;
    let vpd_monthly = all_terra_climate.vpd.get_monthly_values(lat, lon, name)?;
    let ws_monthly = all_terra_climate.ws.get_monthly_values(lat, lon, name)?;

    let humidity_monthly = vap_monthly.iter().zip(vpd_monthly.iter())
        .map(|(vap, vpd)| (vap / (vap + vpd) * 100.0) as u8)
        .collect();

    Some(CityClimate {
        humidity_monthly,
        ppt_monthly,
        srad_monthly,
        tmax_monthly,
        tmin_monthly,
        ws_monthly,
    })
}
