import logging
import point_in_geojson

# Set the logging level based on your needs (e.g., logging.DEBUG)
logging.basicConfig(level=logging.INFO)

print("-> Demonstration of error handling")
try:
    # Demonstrate error handling with malformed JSON
    pig = point_in_geojson.PointInGeoJSON("{")
    assert False
except ValueError:
    logging.exception("Malformed JSON throws ValueError.")

points = [
    # in boundaries
    (7.9743145, 52.2893583),
    # nearby airfield out of boundaries
    (7.973333, 52.286333),
]

print("\n-> Demonstration of point_included(lon, lat)")
with open("field_boundaries.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())
_area_ha = pig.area() / 1e4
print(f"Area of shapes {_area_ha} ha")
for lon, lat in points:
    print(f"Point: ({lon}, {lat}), included: {pig.point_included(lon, lat)}")
    print(f"Closest distance: {pig.closest_distance(lon, lat):.1f} m")

print("\n-> Demonstration of point_included_with_properties(lon, lat)")
with open("manuring_plan.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())
_area_ha = pig.area() / 1e4
print(f"Area of shapes {_area_ha} ha")
for lon, lat in points:
    print(
        f"Point: ({lon}, {lat}), "
        f"properties: {pig.point_included_with_properties(lon, lat)}"
    )
    print(f"Closest distance: {pig.closest_distance(lon, lat):.1f} m")
