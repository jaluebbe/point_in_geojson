use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use geojson::{GeoJson, Geometry, Value};
use geo::{Point};
use geo::algorithm::contains::Contains;

#[pyclass]
struct PointInGeoJSON {
    geojson: GeoJson
}

#[pymethods]
impl PointInGeoJSON {
    #[new]
    pub fn new(value: String) -> PyResult<Self> {
        let geojson_file = value.parse::<GeoJson>();
        match geojson_file {
            Ok(data) => Ok(PointInGeoJSON { geojson: data }),
            Err(_error) => Err(PyValueError::new_err("Invalid GeoJSON string.")),
        }
    }

    fn point_included(&self, lon: f64, lat: f64) -> PyResult<bool> {
        let point = Point::new(lon, lat);
        match self.geojson {
            GeoJson::FeatureCollection(ref ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        if match_geometry(geom, point) {
                            return Ok(true);
                        }
                    }
                }
            },
            GeoJson::Feature(ref feature) => {
                if let Some(ref geom) = feature.geometry {
                    return Ok(match_geometry(geom, point));
                }
            },
            GeoJson::Geometry(ref geom) => {
                return Ok(match_geometry(geom, point));
            },
        }
        Ok(false)
    }

    fn point_included_with_properties(&self, lon: f64, lat: f64) -> PyResult<String> {
        let point = Point::new(lon, lat);
        let mut vector: Vec<geojson::JsonObject> = Vec::new();
        match self.geojson {
            GeoJson::FeatureCollection(ref ctn) => {
                for feature in &ctn.features {
                    if let Some(ref geom) = feature.geometry {
                        if match_geometry(geom, point) {
                            if let Some(properties) = &feature.properties {
                                vector.push(properties.clone());
                            }
                        }
                    }
                }
            },
            GeoJson::Feature(ref feature) => {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry(geom, point) {
                        if let Some(properties) = &feature.properties {
                            vector.push(properties.clone());
                        }
                    }
                }
            },
            GeoJson::Geometry(_) => {},
        }
        let json: String = serde_json::to_string(&vector).unwrap();
        Ok(json)
    }
}

fn match_geometry(geom: &Geometry, point: Point) -> bool {
    match geom.value {
        Value::Polygon(_) | Value::MultiPolygon(_) => {
            let shape: geo_types::Geometry<f64> = geom.try_into().unwrap();
            shape.contains(&point)
        },
        Value::GeometryCollection(ref gc) => {
            for geometry in gc {
                if match_geometry(geometry, point) {
                    return true;
                }
            }
            false
        }
        _ => false
    }
}

#[pymodule]
fn point_in_geojson(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PointInGeoJSON>()?;
    Ok(())
}
