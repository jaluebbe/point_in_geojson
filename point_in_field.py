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

print(
    "\n-> Demonstration of point_included(lon, lat), area() and "
    "closest_distance(lon, lat) on field boundaries."
)
with open("field_boundaries.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())
_area_ha = pig.area() / 1e4
print(f"Area of shapes {_area_ha} ha")
for lon, lat in points:
    print(f"Point: ({lon}, {lat}), included: {pig.point_included(lon, lat)}")
    print(f"Closest distance: {pig.closest_distance(lon, lat):.1f} m")

print(
    "\n-> Demonstration of point_included_with_properties(lon, lat), "
    "area() and closest_distance(lon, lat) on a manuring plan."
)
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

print("\n-> Demonstration of geodesic_distance(lon_1, lat_1, lon_2, lat_2).")
distance = point_in_geojson.geodesic_distance(*points[0], *points[1])
print(f"Distance between the both points: {distance:.1f} m")

print("\n-> Demonstration of geodesic_bearing(lon_1, lat_1, lon_2, lat_2).")
bearing = point_in_geojson.geodesic_bearing(*points[0], *points[1])
print(f"Bearing from the first to the second point: {bearing:.1f} deg")

print(
    "\n-> Demonstration of geodesic_destination(lon_1, lat_1, bearing, distance)."
)
destination = point_in_geojson.geodesic_destination(
    *points[0], bearing, distance
)
print(f"The destination is located at: {destination}")
distance_check = point_in_geojson.geodesic_distance(*destination, *points[1])
print(
    f"The distance to the second point is negligible with: {distance_check:.1} m."
)
