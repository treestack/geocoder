# Reverse Geocoder

Usage:

    docker build -t treestack/geocoder:0 .
    docker run -p 5353:5353 treestack/geocoder:0

    curl "http://localhost:5353/?lat=50.9&lng=7.2"  

Example response:

    {
        "geometry": {
            "coordinates": [
                7.1817498207092285,
                50.895591735839844
            ],
            "type": "Point"
        },
        "id": 32025,
        "name": "Rosrath",
        "properties": {
            "admin1": "North Rhine-Westphalia",
            "admin2": "Regierungsbezirk Koln",
            "country": "DE",
            "distanceToQuery": 1370.523193359375
        },
        "type": "Feature"
    }

