# point_in_geojson
The purpose of this repository is to monitor the relation of GPS coordinates to GeoJSON shapes. This includes to check if a point is inside any polygon or to receive the properties of all matching polygons. 

Some helper functions allow to calculate the area of the shapes, the distance between two points, the distance between a point and the closest shape, the geodesic bearing between two points. Using a point, bearing and distance, the destination of another point could be calculated. The GeoJSON shape being used could be returned as dict.

For usage examples look into the tests folder or point_in_field.py .

As compiling Rust on a Raspberry Pi may take a really long time, I provide some precompiled wheels on pypi.org.

Just try ```pip install point-in-geojson-jaluebbe```
