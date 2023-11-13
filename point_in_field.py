import logging
import json
import point_in_geojson

try:
    pig = point_in_geojson.PointInGeoJSON("{")
    assert False
except ValueError:
    logging.exception("malformed JSON throws ValueError.")
print("-> Test of Error handling finished.")

points = [
    # in boundaries
    (
        7.9743145,
        52.2893583,
        True,
        [{"INDEX": 0.4275, "RATE": 115, "V22RATE": "0.92"}],
    ),
    # nearby airfield out of boundaries
    (7.973333, 52.286333, False, []),
]

with open("field_boundaries.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())
for _point in points:
    _lon, _lat = _point[:2]
    _in_boundaries = _point[2]
    assert pig.point_included(_lon, _lat) == _in_boundaries
print("-> Test of point_included(lon, lat) passed.")

with open("manuring_plan.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())
for _point in points:
    _lon, _lat = _point[:2]
    _properties = _point[3]
    assert (
        json.loads(pig.point_included_with_properties(_lon, _lat))
        == _properties
    )
print("-> Test of point_included_with_properties(lon, lat) passed.")
