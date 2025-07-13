use std::{fmt::Debug};

use common::util::round_0_1_and_assert_finite;
use preprocessing::{get_closest_index,  iterate_increasing_squares};

pub trait TerraVal: netcdf::NcTypeDescriptor + Copy + Eq {
    fn read_attr_value(var: &netcdf::Variable, attr_name: &str) -> Self;
    fn to_f32(value: Self) -> f32;
}

macro_rules! read_attr_as {
    ($var:expr, $attr_name:expr, $netcdf_variant:ident, $type:ty) => {
        match $var.attribute($attr_name).unwrap().value().unwrap() {
            netcdf::AttributeValue::$netcdf_variant(val) => val as $type,
            _ => panic!("{}.{} is not {}", $var.name(), $attr_name, stringify!($type)),
        }
    };
}

impl TerraVal for i16 {
    fn read_attr_value(var: &netcdf::Variable, attr_name: &str) -> i16 { read_attr_as!(var, attr_name, Short, i16) }
    fn to_f32(value: Self) -> f32 { value as f32 }
}

impl TerraVal for i32 {
    fn read_attr_value(var: &netcdf::Variable, attr_name: &str) -> i32 { read_attr_as!(var, attr_name, Int, i32) }
    fn to_f32(value: Self) -> f32 { value as f32 }
}

pub struct TerraClimateData<T: TerraVal> {
    var_name: String,
    lat_values: Vec<f64>,
    lon_values: Vec<f64>,
    missing_value: T,
    scale_factor: f32,
    add_offset: f32,
    var_values: Vec<T>,
}

impl<T: TerraVal> TerraClimateData<T> {
    pub fn new(var_name: &str) -> TerraClimateData<T> {
        let file_name = format!("TerraClimate19912020_{}.nc", var_name);
        let file = netcdf::open(common::util::get_data_in_dir().join(&file_name)).unwrap();

        let get_all_values_f64 = |var_name: &str| {
            file.variable(var_name).unwrap().get_values::<f64, _>(..).unwrap()
        };

        let lat_values = get_all_values_f64("lat");
        let lon_values = get_all_values_f64("lon");

        let var = file.variable(var_name).unwrap();
        let missing_value = T::read_attr_value(&var, "missing_value");
        let scale_factor = read_attr_as!(&var, "scale_factor", Double, f32);
        let add_offset = read_attr_as!(&var, "add_offset", Double, f32);
        let var_values = var.get_values::<T, _>(..).unwrap();

        let terra_climate_data = TerraClimateData::<T> {
            var_name: var_name.to_owned(),
            lat_values,
            lon_values,
            missing_value,
            scale_factor,
            add_offset,
            var_values,
        };

        eprintln!("{:?}", terra_climate_data);

        terra_climate_data
    }

    pub fn get_monthly_values(&self, lat: f64, lon: f64, city: &str) -> Option<[f32; 12]> {
        let lat_index = get_closest_index(&self.lat_values, lat);
        let lon_index = get_closest_index(&self.lon_values, lon);

        let mut monthly_values = [0_f32; 12];
        for month in 0..12 {
            let val = self.get_closest_value(month, lat_index, lon_index);
            if val.is_none() {
                eprintln!("{} at {:?}: not found \"{}\"", city, (lat, lon), self.var_name);
                return None;
            }
            monthly_values[month] = val.unwrap();
        }

        Some(monthly_values)
    }

    fn get_closest_value(&self, month: usize, lat_index: usize, lon_index: usize) -> Option<f32> {
        iterate_increasing_squares(lon_index, lat_index, 5, self.lon_values.len(), self.lat_values.len())
            .find_map(|(lon_i, lat_i)| self.get_value(month, lat_i, lon_i))
    }

    fn get_value(&self, month: usize, lat_index: usize, lon_index: usize) -> Option<f32> {
        let flat_index = month * self.lat_values.len() * self.lon_values.len()
            + lat_index * self.lon_values.len()
            + lon_index;
        let raw = self.var_values[flat_index];
        if raw == self.missing_value {
            None
        } else {
            Some(round_0_1_and_assert_finite(T::to_f32(raw) * self.scale_factor + self.add_offset))
        }
    }
}

impl<T: TerraVal> Debug for TerraClimateData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerraClimateData")
            .field("var_name", &self.var_name)
            .field("lat_values.len()", &self.lat_values.len())
            .field("lon_values.len()", &self.lon_values.len())
            .field("scale_factor", &self.scale_factor)
            .field("add_offset", &self.add_offset)
            .field("var_values.len()", &self.var_values.len())
            .finish()
    }
}
