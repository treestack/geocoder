# Reverse Geocoder

This is a simple and very lightweight reverse geocoder for offline use. It's implemented in Rust and uses a k-d-tree to find the closest cities for a given coordinate.
The recommended data set from GeoNames contains ~200,000 cities with a population of at least 500, but smaller datasets work fine if you require less accuracy.

Development is sponsored by [Treestack GmbH](https://treestack.de).

## How to use the docker image

### Prerequisites

The image only contains a demo data set of a few cities. We chose not to include the full data file in the docker image, so both can be updated independently.

Download a `cities500.zip` from [GeoNames](http://download.geonames.org/export/dump/) and unpack it. You should now have a `cities500.txt`. 
If you require less accuracy and/or have limited resources, you can download any other `citiesN.txt` instead.

### Run from command line:

    $ docker run \
        -p 5353:5353 \
        -v $(pwd)/cities500.txt:/cities.txt \
        -e GEOCODER_BIND_ADDRESS=0.0.0.0:5353 \
        ghcr.io/treestack/geocoder:master

### Run with Docker compose

`docker-compose.yml`:

    version: "3.9"

    services:
      geocoder:
        image: ghcr.io/treestack/geocoder:master
        ports:
          - "5353:5353"
        environment:
          GEOCODER_BIND_ADDRESS: 0.0.0.0:5353
        volumes:
          - "./cities500.txt:/cities.txt"

### Run with Kubernetes (experimental)

There's a helm chart in the `deploy` directory, but it's haphazardly stitched together. 
Please feel free to create a PR if you have any suggestions.

    helm install <name> deploy 

## Configuration

You can configure the application with the following environment variables:

| Parameter                  | Description                             | Default        |
|----------------------------|-----------------------------------------|----------------|
| GEOCODER_BIND_ADDRESS      | Bind address                            | 127.0.0.1:5353 |
| GEOCODER_LOGLEVEL          | Log level                               | INFO           |
| GEOCODER_DATA_FILE         | Data file name                          | ./cities.txt   |
| GEOCODER_WATCH_FOR_CHANGES | Reload geocoder when data file changes* | true           |
| GEOCODER_ALLOW_ORIGIN      | CORS Access-Control-Allow-Origin header | *              |

\* Incredibly unreliable when the datafile is mounted as a docker volume.

## Usage

### Example call

    curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"

### Request parameters

The microservice listens to all GET requests and supports the following query parameters:

| Parameter | Description                                               | Required | Example  |
|-----------|-----------------------------------------------------------|----------|----------|
| **lat**   | Latitude (WGS84, decimal)                                 | Yes      | -48.875  |
| **lng**   | Longitude (WGS84, decimal)                                | Yes      | -123.392 |
| results   | Number of results, integer, defaults to `1`               | No       | 10       |
| details   | Include details in response, boolean, defaults to `false` | No       | true     |

### Response

The response is a valid GeoJSON `FeatureCollection`. The feature's `id` is added as [foreign members](https://www.rfc-editor.org/rfc/rfc7946#section-6.1). 
The additional properties always includes the 'title' and distance to the given coordinates. 
Optionally you can add most columns from the geonames dataset by setting the `details` parameter to `true`:

| Property            | Description                                                                                     |  
|---------------------|-------------------------------------------------------------------------------------------------|
| **title**           | The city's name                                                                                 |
| **distanceToQuery** | Approx. distance to given coordinates in kilometres (assuming earth radius of exactly 6371 km). |
| admin1Code          |                                                                                                 |
| admin2Code          |                                                                                                 |
| admin3Code          |                                                                                                 |
| admin4Code          |                                                                                                 |
| countryCode         | ISO-3166 2-letter country code                                                                  |
| cc2                 | alternative country codes                                                                       |
| dem                 | digital elevation model, srtm3 or gtopo30                                                       |
| elevation           | elevation in metres                                                                             |
| featureCode         | Feature code. For a complete list, check [here](http://www.geonames.org/export/codes.html).     |
| modificationDate    | Last modification date                                                                          |
| population          | Population                                                                                      |
| timezone            | IANA timezone id                                                                                |

#### Example

	{
		"type": "FeatureCollection",
		"features": [
			{
				"geometry": {
					"coordinates": [
						78.25628662109375,
						28.95911979675293
					],
					"type": "Point"
				},
				"id": 1272983,
				"properties": {
					"distanceToQuery": 6,
					"title": "Dhanaura"
				},
				"type": "Feature"
			},
			{
				"geometry": {
					"coordinates": [
						78.23455810546875,
						28.92694091796875
					],
					"type": "Point"
				},
				"id": 1278036,
				"properties": {
					"distanceToQuery": 8,
					"title": "Bachhraon"
				},
				"type": "Feature"
			}
		]
	}

## Resource use and Performance

The final docker image has a size of only 8 MB, memory usage depends on the used data set:

| Dataset         | Cities  | Approx. mem. usage |
|-----------------|---------|--------------------|
| cities500.txt   | 199,606 | ~160 MiB           |
| cities5000.txt  | 53,268  | ~45 MiB            |
| cities15000.txt | 26,457  | ~25 MiB            |

With `cities500.txt`, response time is consistently < 10 ms, measured on a M1 mac:

    $ hyperfine --warmup 3 'curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1"'
    Benchmark 1: curl "http://localhost:5353?lat=-48.875486&lng=-123.392519&results=1&details=true"
    Time (mean ± σ):       5.8 ms ±   0.9 ms    [User: 1.5 ms, System: 1.9 ms]
    Range (min … max):     4.1 ms …   9.3 ms    374 runs

## Development

### Run locally

    $ cargo run web

### Build local docker image

    docker build -t treestack/geocoder:0 .
