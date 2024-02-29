import logging
import json
import pytest
import point_in_geojson


def test_error_handling():
    with pytest.raises(ValueError):
        point_in_geojson.PointInGeoJSON("{")
    logging.info("Test of Error handling finished.")


def test_point_included():
    points = [
        (
            7.9743145,
            52.2893583,
            True,
            [{"INDEX": 0.4275, "RATE": 115, "V22RATE": "0.92"}],
        ),
        (7.973333, 52.286333, False, []),
    ]

    with open("field_boundaries.json") as f:
        pig = point_in_geojson.PointInGeoJSON(f.read())

    for lon, lat, in_boundaries, _ in points:
        assert pig.point_included(lon, lat) == in_boundaries
    logging.info("Test of point_included(lon, lat) passed.")


def test_point_included_with_properties():
    points = [
        (
            7.9743145,
            52.2893583,
            [{"INDEX": 0.4275, "RATE": 115, "V22RATE": "0.92"}],
        ),
        (7.973333, 52.286333, []),
    ]

    with open("manuring_plan.json") as f:
        pig = point_in_geojson.PointInGeoJSON(f.read())

    for lon, lat, properties in points:
        assert pig.point_included_with_properties(lon, lat) == properties
    logging.info("Test of point_included_with_properties(lon, lat) passed.")


def test_area_calculation():
    with open("field_boundaries.json") as f:
        pig = point_in_geojson.PointInGeoJSON(f.read())
        area_ha = pig.area() / 1e4
        assert area_ha > 0
        assert area_ha == 8.4747
    logging.info("Test of area() passed.")


def test_closest_distance():
    points = [
        (7.9743145, 52.2893583, 0.0),
        (7.973333, 52.286333, 210.5),
    ]
    with open("field_boundaries.json") as f:
        pig = point_in_geojson.PointInGeoJSON(f.read())

    for lon, lat, closest_distance in points:
        assert round(pig.closest_distance(lon, lat), 1) == closest_distance
    logging.info("Test of closest_distance(lon, lat) passed.")


def test_distance_calculation():
    points = [
        ((7.9743145, 52.2893583, 7.973333, 52.286333), 343.2),
        ((7.9743145, 52.2893583, 7.9743145, 52.2893583), 0.0),
    ]
    for coordinates, distance in points:
        assert (
            round(point_in_geojson.geodesic_distance(*coordinates), 1)
            == distance
        )
    logging.info(
        "Test of geodesic_distance(lon_1, lat_1, lon_2, lat_2) passed."
    )


def test_geodesic_bearing():
    points = [
        ((7.9743145, 52.2893583, 7.973333, 52.286333), -168.7),
        ((7.9743145, 52.2893583, 7.9743145, 52.2893583), 180),
    ]
    for coordinates, bearing in points:
        assert (
            round(point_in_geojson.geodesic_bearing(*coordinates), 1) == bearing
        )
    logging.info("Test of geodesic_bearing(lon_1, lat_1, lon_2, lat_2) passed.")


def test_geodesic_destination():
    point_1 = (7.9743145, 52.2893583)
    point_2 = (7.973333, 52.286333)
    distance = point_in_geojson.geodesic_distance(*point_1, *point_2)
    bearing = point_in_geojson.geodesic_bearing(*point_1, *point_2)
    destination = point_in_geojson.geodesic_destination(
        *point_1, bearing, distance
    )
    distance_check = point_in_geojson.geodesic_distance(*destination, *point_1)
    assert distance_check < 1.0e9
    logging.info(
        "Test of geodesic_destination(lon_1, lat_1, bearing, distance) passed."
    )


def test_to_dict():
    with open("field_boundaries.json") as f:
        boundaries_dict = json.load(f)
    with open("field_boundaries.json") as f:
        pig = point_in_geojson.PointInGeoJSON(f.read())
    assert boundaries_dict == pig.to_dict()
    logging.info("Test of to_dict() passed.")
