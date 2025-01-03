
use geo::{Bearing, Destination, Distance, Geodesic, Point};
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::contains::Contains;
use geo::algorithm::geodesic_area::GeodesicArea;
use geojson::{GeoJson, Geometry, Value};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
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
        let py_dict = pythonize(py, &self.geojson).unwrap();
        Ok(py_dict.into())
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

    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let py_dict = pythonize(py, &self.geojson).unwrap();
        Ok(py_dict.into())
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
                    Geodesic::distance(point, *closest_point)
                },
                geo::Closest::SinglePoint(closest_point) => {
                    Geodesic::distance(point, *closest_point)
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

#[pyfunction]
fn geodesic_distance(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> PyResult<f64> {
    let point1 = Point::new(lon1, lat1);
    let point2 = Point::new(lon2, lat2);
    Ok(Geodesic::distance(point1, point2))
}

#[pyfunction]
fn geodesic_destination(lon: f64, lat: f64, bearing: f64, distance: f64) -> PyResult<(f64, f64)> {
    let point = Point::new(lon, lat);
    let destination = Geodesic::destination(point, bearing, distance);
    Ok((destination.x(), destination.y()))
}

#[pyfunction]
fn geodesic_bearing(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> PyResult<f64> {
    let point1 = Point::new(lon1, lat1);
    let point2 = Point::new(lon2, lat2);
    Ok(Geodesic::bearing(point1, point2))
}

#[pymodule]
fn point_in_geojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PointInGeoJSON>()?;
    m.add_function(wrap_pyfunction!(geodesic_distance, m)?)?;
    m.add_function(wrap_pyfunction!(geodesic_destination, m)?)?;
    m.add_function(wrap_pyfunction!(geodesic_bearing, m)?)?;
    Ok(())
}
