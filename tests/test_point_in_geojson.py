import logging
import point_in_geojson
import pytest


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


if __name__ == "__main__":
    test_error_handling()
    test_point_included()
    test_point_included_with_properties()
