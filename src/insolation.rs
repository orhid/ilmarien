use crate::imaging::cartography as crt;
use crate::imaging::hexagonos::{Gon, PreGon};
use geo_types::Coordinate;
//use log::debug;
use log::info;
use nalgebra::Vector3;
use rayon::prelude::*;
use std::f64::consts::TAU;

const MAX_WATCH: usize = 16;

fn insolation_curve(value: f64) -> f64 {
    1.8 * value - 3.2
}

fn insolation_watch_curve(value: f64) -> f64 {
    //debug!("{:?}", value - 0.5);
    value - 0.5
}

fn vector_elevation(point: &Coordinate<f64>, elevation: f64) -> Vector3<f64> {
    let cpoint = point.centre();
    Vector3::new(cpoint.x, cpoint.y, elevation)
}

/* daily */

fn insolation_calculate_sun(point: &Coordinate<f64>, sun: &Coordinate<f64>) -> f64 {
    //turns out, the influence of both elevation and slope is negligable
    let sunward = vector_elevation(&sun, 1.0) - vector_elevation(&point, 0.0);
    sunward.norm().powi(-2) * sunward.dot(&Vector3::new(0.0, 0.0, 1.0)) / sunward.norm()
}

fn insolation_calculate_point(point: &Coordinate<f64>) -> f64 {
    let detail = 3;
    let suns = point.find().ball(detail);
    insolation_curve(
        suns.iter()
            .map(|sun| Coordinate {
                x: sun.x as f64,
                y: sun.y as f64,
            })
            .map(|sun| insolation_calculate_sun(&point, &sun))
            .sum::<f64>(),
    )
}

pub fn insolation_calculate(resolution: usize) -> crt::Brane {
    //! calculate insolation – the amount of radiation reaching the surface over a single day

    info!("calculating insolation map");

    let mut brane = crt::new("insolation".to_string(), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| insolation_calculate_point(&point))
            .collect(),
    );
    brane
}

/* watchly */

fn encircle(watch: usize) -> f64 {
    TAU * watch as f64 / (1.0 * MAX_WATCH as f64)
}

fn insolation_watch_calculate_sun(
    point: &Coordinate<f64>,
    sun: &Coordinate<f64>,
    watch: usize,
) -> f64 {
    let sunward = vector_elevation(&sun, 1.0) - vector_elevation(&point, 0.0);
    let angle = sunward[1].atan2(sunward[0]);
    (angle * 3.0 + encircle(watch)).sin().max(0.0).powf(0.72)
        * sunward.norm().powi(-2)
        * sunward.dot(&Vector3::new(0.0, 0.0, 1.0))
        / sunward.norm()
}

fn insolation_watch_calculate_point(point: &Coordinate<f64>, watch: usize) -> f64 {
    let detail = 8;
    let suns = point.find().ball(detail);
    insolation_watch_curve(
        suns.iter()
            .map(|sun| Coordinate {
                x: sun.x as f64,
                y: sun.y as f64,
            })
            .map(|sun| insolation_watch_calculate_sun(&point, &sun, watch))
            .sum::<f64>(),
    )
}

pub fn insolation_watch_calculate(resolution: usize, watch: usize) -> crt::Brane {
    //! calculate insolation – the amount of radiation reaching the surface over a single watch

    info!("calculating insolation map at watch {}", watch);

    let mut brane = crt::new(format!("insolation-{}", watch), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| insolation_watch_calculate_point(&point, watch))
            .collect(),
    );
    brane
}
