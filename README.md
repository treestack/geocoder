# Reverse Geocoder

Simple, lightweight reverse geocoder. Ignores boundaries and just returns the closest (known) city for the given 
coordinates.

## Configuration

| Parameter    | Description    | Default         |
|--------------|----------------|-----------------|
| BIND_ADDRESS | Bind address   | 0.0.0.0:5353    |
| LOGLEVEL     | Log level      | DEBUG           |
| DATA_FILE    | Data file name | ./cities500.txt |

## Requirements

Requires `cities500.txt` from https://geonames.org.

Download here: http://download.geonames.org/export/dump/cities500.zip and replace the placeholder file.

## Build

    docker build -t treestack/geocoder:0 .
    docker run -p 5353:5353 treestack/geocoder:0

## Usage

    curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"  

### Parameters:

| Parameter | Description                                             | Required | Example  |
|-----------|---------------------------------------------------------|----------|----------|
| lat       | Latitude (WGS84, decimal)                               | Yes      | -48.875  |
| lng       | Longitude (WGS84, decimal)                              | Yes      | -123.392 |
| results   | Number of results, integer, defaults to 1               | No       | 10       |
| details   | Include details in response, boolean, defaults to false | No       | true     |

### Example response

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
                "distanceToQuery": 10396437,
                "elevation": null,
                "featureCode": "PPL",
                "modificationDate": "2022-10-05",
                "population": 371244,
                "timezone": "Africa/Harare"
            },
            "type": "Feature"
        }
    ]
