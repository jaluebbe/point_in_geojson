use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use geojson::{GeoJson, Geometry, Value};
use geo::{Point};
use geo::algorithm::contains::Contains;
use geo::algorithm::geodesic_area::GeodesicArea;
use geo::algorithm::geodesic_distance::GeodesicDistance;
use geo::algorithm::closest_point::ClosestPoint;
use pythonize::pythonize;

#[pyclass]
struct PointInGeoJSON {
    geojson: GeoJson
}

#[pymethods]
impl PointInGeoJSON {
    #[new]
    pub fn new(value: String) -> PyResult<Self> {
        let geojson_file = value.parse::<GeoJson>().map_err(|err| PyValueError::new_err(format!("Invalid GeoJSON string: {}", err)))?;
        Ok(PointInGeoJSON { geojson: geojson_file })
    }

    fn point_included(&self, lon: f64, lat: f64) -> PyResult<bool> {
        let point = Point::new(lon, lat);
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                Ok(ctn.features.iter().any(|feature| {
                    feature.geometry.as_ref().map_or(false, |geom| match_geometry_and_point(geom, point))
                }))
            },
            GeoJson::Feature(feature) => {
                Ok(feature.geometry.as_ref().map_or(false, |geom| match_geometry_and_point(geom, point)))
            },
            GeoJson::Geometry(geom) => {
                Ok(match_geometry_and_point(geom, point))
            },
        }
    }

    fn point_included_with_properties(&self, py: Python<'_>, lon: f64, lat: f64) -> PyResult<Py<PyAny>> {
        let point = Point::new(lon, lat);
        let mut vector: Vec<geojson::JsonObject> = Vec::new();
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        if match_geometry_and_point(geom, point) {
                            if let Some(properties) = &feature.properties {
                                vector.push(properties.clone());
                            }
                        }
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry_and_point(geom, point) {
                        if let Some(properties) = &feature.properties {
                            vector.push(properties.clone());
                        }
                    }
                }
            },
            GeoJson::Geometry(_) => {},
        }
        Ok(pythonize(py, &vector).unwrap())
    }

    fn area(&self) -> PyResult<f64> {
        let mut total_area = 0.0;
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        total_area += match_polygon_area(geom);
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(ref geom) = feature.geometry {
                    total_area += match_polygon_area(geom);
                }
            },
            GeoJson::Geometry(geom) => {
                total_area += match_polygon_area(geom);
            }
        }
        Ok(total_area.round())
    }

    fn closest_distance(&self, lon: f64, lat: f64) -> PyResult<f64> {
        let point = Point::new(lon, lat);
        let mut min_distance = f64::INFINITY;
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        min_distance = min_distance.min(match_geometry_distance(geom, point));
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(ref geom) = feature.geometry {
                    min_distance = min_distance.min(match_geometry_distance(geom, point));
                }
            },
            _ => {}
        }
        Ok(min_distance)
    }
}

fn match_geometry_and_point(geom: &Geometry, point: Point) -> bool {
    match &geom.value {
        Value::Polygon(_) | Value::MultiPolygon(_) => {
            let shape: geo_types::Geometry<f64> = geom.try_into().unwrap();
            shape.contains(&point)
        },
        Value::GeometryCollection(gc) => {
            gc.iter().any(|geometry| match_geometry_and_point(geometry, point))
        }
        _ => false
    }
}

fn match_polygon_area(geom: &Geometry) -> f64 {
    match &geom.value {
        Value::Polygon(_) | Value::MultiPolygon(_) => {
            let shape: geo_types::Geometry<f64> = geom.try_into().unwrap();
            shape.geodesic_area_signed().abs()
        },
        _ => 0.0
    }
}

fn match_geometry_distance(geom: &Geometry, point: Point<f64>) -> f64 {
    match &geom.value {
        Value::Polygon(_) | Value::MultiPolygon(_) => {
            let shape: geo_types::Geometry<f64> = geom.try_into().unwrap();
            let closest_result = shape.closest_point(&point);
            match &closest_result {
                geo::Closest::Intersection(closest_point) => {
                    point.geodesic_distance(&closest_point)
                },
                geo::Closest::SinglePoint(closest_point) => {
                    point.geodesic_distance(&closest_point)
                },
                _ => {
                    f64::INFINITY
                },
            }
        },
        Value::GeometryCollection(gc) => {
            gc.iter().fold(f64::INFINITY, |min_distance, geometry| {
                let distance = match_geometry_distance(geometry, point);
                min_distance.min(distance)
            })
        },
        _ => f64::INFINITY
    }
}

#[pymodule]
fn point_in_geojson(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PointInGeoJSON>()?;
    Ok(())
}
