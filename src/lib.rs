
use geo::{Bearing, Destination, Distance, Geodesic, Point};
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::contains::Contains;
use geo::algorithm::geodesic_area::GeodesicArea;
use geojson::{GeoJson, Geometry, Value};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::borrow::Borrow;
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
        let py_dict = pythonize(py, &vector).unwrap();
        Ok(py_dict.into())
    }

    fn point_included_with_features(&self, py: Python<'_>, lon: f64, lat: f64) -> PyResult<Py<PyAny>> {
        let point = Point::new(lon, lat);
        let mut vector: Vec<geojson::Feature> = Vec::new();
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        if match_geometry_and_point(geom, point) {
                            vector.push(feature.clone());
                        }
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry_and_point(geom, point) {
                        vector.push(feature.clone());
                    }
                }
            },
            GeoJson::Geometry(_) => {},
        }
        let py_dict = pythonize(py, &vector).unwrap();
        Ok(py_dict.into())
    }

    #[pyo3(signature = (key, value, match_type=None))]
    fn features_with_property_str(&self, py: Python<'_>, key: String, value: String, match_type: Option<&str>) -> PyResult<Py<PyAny>> {
        let match_type = match_type.unwrap_or("equal");
        let vector = self.filter_features_by_property_str(&key, &value, match_type);
        let py_dict = pythonize(py, &vector).unwrap();
        Ok(py_dict.into())
    }

    fn features_with_property_int(&self, py: Python<'_>, key: String, value: i64) -> PyResult<Py<PyAny>> {
        let value: serde_json::Value = value.into();
        let vector = self.filter_features_by_property(&key, &value);
        let py_dict = pythonize(py, &vector).unwrap();
        Ok(py_dict.into())
    }

    fn features_with_property_float(&self, py: Python<'_>, key: String, value: f64) -> PyResult<Py<PyAny>> {
        let value: serde_json::Value = value.into();
        let vector = self.filter_features_by_property(&key, &value);
        let py_dict = pythonize(py, &vector).unwrap();
        Ok(py_dict.into())
    }
    #[pyo3(signature = (key, value, match_type=None))]
    fn features_with_property(&self, key: String, value: Py<PyAny>, match_type: Option<&str>) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            let value = value.borrow();
            if let Ok(value_str) = value.extract::<String>(py) {
                return self.features_with_property_str(py, key, value_str, match_type);
            } else if let Ok(value_int) = value.extract::<i64>(py) {
                return self.features_with_property_int(py, key, value_int);
            } else if let Ok(value_float) = value.extract::<f64>(py) {
                return self.features_with_property_float(py, key, value_float);
            } else {
                Err(PyValueError::new_err("Unsupported value type"))
            }
        })
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

impl PointInGeoJSON {
    fn filter_features_by_property(&self, key: &str, value: &serde_json::Value) -> Vec<geojson::Feature> {
        let mut vector: Vec<geojson::Feature> = Vec::new();
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(properties) = &feature.properties {
                        if let Some(prop_value) = properties.get(key) {
                            if prop_value == value {
                                vector.push(feature.clone());
                            }
                        }
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(properties) = &feature.properties {
                    if let Some(prop_value) = properties.get(key) {
                        if prop_value == value {
                            vector.push(feature.clone());
                        }
                    }
                }
            },
            GeoJson::Geometry(_) => {},
        }
        vector
    }

    fn filter_features_by_property_str(&self, key: &str, value: &str, match_type: &str) -> Vec<geojson::Feature> {
        let mut vector: Vec<geojson::Feature> = Vec::new();
        match &self.geojson {
            GeoJson::FeatureCollection(ctn) => {
                for feature in &ctn.features {
                    if let Some(properties) = &feature.properties {
                        if let Some(prop_value) = properties.get(key) {
                            if let Some(prop_str) = prop_value.as_str() {
                                match match_type {
                                    "equal" if prop_str == value => vector.push(feature.clone()),
                                    "starts_with" if prop_str.starts_with(value) => vector.push(feature.clone()),
                                    "contains" if prop_str.contains(value) => vector.push(feature.clone()),
                                    "ends_with" if prop_str.ends_with(value) => vector.push(feature.clone()),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            },
            GeoJson::Feature(feature) => {
                if let Some(properties) = &feature.properties {
                    if let Some(prop_value) = properties.get(key) {
                        if let Some(prop_str) = prop_value.as_str() {
                            match match_type {
                                "equal" if prop_str == value => vector.push(feature.clone()),
                                "starts_with" if prop_str.starts_with(value) => vector.push(feature.clone()),
                                "contains" if prop_str.contains(value) => vector.push(feature.clone()),
                                "ends_with" if prop_str.ends_with(value) => vector.push(feature.clone()),
                                _ => {}
                            }
                        }
                    }
                }
            },
            GeoJson::Geometry(_) => {},
        }
        vector
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
