# Dependencies

## [Geonames](https://www.geonames.org/) files

From [dumps](https://download.geonames.org/export/dump/):

* `admin1CodesASCII.txt`
* `cities500.txt`
* `cities5000.txt`
* `cities15000.txt`
* `countryInfo.txt`

## [ClimatologyLab](https://www.climatologylab.org/terraclimate.html) files

From [TerraClimate catalog](http://thredds.northwestknowledge.net:8080/thredds/catalog/TERRACLIMATE_ALL/summaries/catalog.html):

For all files select access type 2 - HTTPServer

* `TerraClimate19912020_ws.nc`
* `TerraClimate19912020_vpd.nc`
* `TerraClimate19912020_vap.nc`
* `TerraClimate19912020_tmin.nc`
* `TerraClimate19912020_tmax.nc`
* `TerraClimate19912020_srad.nc`
* `TerraClimate19912020_ppt.nc`

Files viewer: [Panoply](https://www.giss.nasa.gov/tools/panoply/download/)

## [NetCDF](https://docs.rs/crate/netcdf/latest) dependency

See the [documentation](https://docs.rs/crate/netcdf/latest) -- requires the `libnetcdf` installed in the system. In OS X works via `brew install netcdf`.
