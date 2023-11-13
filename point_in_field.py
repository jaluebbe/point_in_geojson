import point_in_geojson

with open("field_boundaries.json") as f:
    pig = point_in_geojson.PointInGeoJSON(f.read())

for _point in [
    (7.9743145, 52.2893583),  # in boundaries
    (7.973333, 52.286333),  # nearby airfield out of boundaries
]:
    print(_point, pig.point_included(*_point))
