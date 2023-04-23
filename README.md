[![Docker](https://github.com/treestack/geocoder/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/treestack/geocoder/actions/workflows/docker-publish.yml)

# Reverse Geocoder

Simple, lightweight reverse geocoder. Final Docker image is around 40 MB including data (subject to change). 
Ignores boundaries and just returns the closest known city, including timezone and population data for the given 
coordinates. 

## Performance

With geonames' full dataset of ~200.000 cities, response time is fairly fast (measured on a M1 mac, application running in a Docker container): 

    $ hyperfine --warmup 3 'curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"'

    Benchmark 1: curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"
    Time (mean ± σ):       9.0 ms ±   2.5 ms    [User: 1.8 ms, System: 2.5 ms]
    Range (min … max):     6.0 ms …  20.4 ms    330 runs

## Configuration

You can configure the application with the following environment variables:

| Parameter                  | Description                                                                                          | Default         |
|----------------------------|------------------------------------------------------------------------------------------------------|-----------------|
| GEOCODER_BIND_ADDRESS      | Bind address                                                                                         | 127.0.0.1:5353  |
| GEOCODER_LOGLEVEL          | Log level                                                                                            | INFO            |
| GEOCODER_DATA_FILE         | Data file name                                                                                       | ./cities500.txt |
| GEOCODER_QUOTA_BURST_SIZE  | Rate limiter config, see [Governor docs](https://docs.rs/governor/latest/governor/struct.Quota.html) | 10              |
| GEOCODER_QUOTA_INTERVAL    |                                                                                                      | 1000            |
| GEOCODER_WATCH_FOR_CHANGES | Reload geocoder when data file changes.                                                              | true            |

## Requirements

Requires `cities500.txt` from https://geonames.org.

Download here: http://download.geonames.org/export/dump/cities500.zip and replace the placeholder file.

## Build

    docker build -t treestack/geocoder:0 .
    docker run \
        -p 5353:5353 \
        -e GEOCODER_BIND_ADDRESS=0.0.0.0:5353 \
        treestack/geocoder:0

## Usage

    curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"  

### Parameters:

| Parameter | Description                                             | Required | Example  |
|-----------|---------------------------------------------------------|----------|----------|
| lat       | Latitude (WGS84, decimal)                               | Yes      | -48.875  |
| lng       | Longitude (WGS84, decimal)                              | Yes      | -123.392 |
| results   | Number of results, integer, defaults to 1               | No       | 10       |
| details   | Include details in response, boolean, defaults to false | No       | true     |

### Response

The response is valid GeoJSON, `id` and `name` are added as [foreign members](https://www.rfc-editor.org/rfc/rfc7946#section-6.1). The additional properties always includes the distance to the given coordinates and optionally most columns from the geonames dataset:

| Property         | Description                                                                                     |  
|------------------|-------------------------------------------------------------------------------------------------|
| distanceToQuery  | Approx. distance to given coordinates in kilometres (assuming earth radius of exactly 6371 km). |
| admin1Code       |                                                                                                 |
| admin2Code       |                                                                                                 |
| admin3Code       |                                                                                                 |
| admin4Code       |                                                                                                 |
| countryCode      | ISO-3166 2-letter country code                                                                  |
| cc2              | alternative country codes                                                                       |
| dem              | digital elevation model, srtm3 or gtopo30                                                       |
| elevation        | elevation in metres                                                                             |
| featureCode      | Feature code. For a complete list, check [here](http://www.geonames.org/export/codes.html).     |
| modificationDate | Last modification date                                                                          |
| population       | Population                                                                                      |
| timezone         | IANA timezone id                                                                                |

#### Example

    [
        {
            "geometry": {
                "coordinates": [
                    31.075550079345703,
                    -18.012739181518555
                ],
                "type": "Point"
            },
            "id": 1106542,
            "name": "Chitungwiza",
            "properties": {
                "admin1Code": "10",
                "admin2Code": "",
                "admin3Code": "",
                "admin4Code": "",
                "cc2": "",
                "countryCode": "ZW",
                "dem": "1435",
                "distanceToQuery": 10396,
                "elevation": null,
                "featureCode": "PPL",
                "modificationDate": "2022-10-05",
                "population": 371244,
                "timezone": "Africa/Harare"
            },
            "type": "Feature"
        }
    ]
