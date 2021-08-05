use crate::climate::surface::{decode, Surface};
use crate::imaging::{
    cartography::Brane,
    hexagonos::{Gon, PreGon},
};
use geo_types::Coordinate;
use log::info;
use nalgebra::Vector3;
use rayon::prelude::*;
use std::f64::consts::TAU;

/* # insolation */

fn vector_elevation(point: &Coordinate<f64>, elevation: f64) -> Vector3<f64> {
    let cpoint = point.centre();
    Vector3::new(cpoint.x, cpoint.y, elevation)
}

/* ## daily */

fn insolation_curve(value: f64) -> f64 {
    1.8 * value - 3.2
}

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

/// calculate insolation – the amount of radiation reaching the surface over a single day
pub fn insolation_calculate(resolution: usize) -> Brane<f64> {
    info!("calculating insolation map");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| insolation_calculate_point(&point))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "insolation".to_string();
    brane
}

/* ## watchly */

const MAX_WATCH: usize = 16;

fn encircle(watch: usize) -> f64 {
    TAU * watch as f64 / (1.0 * MAX_WATCH as f64)
}

fn insolation_watch_curve(value: f64) -> f64 {
    value - 0.5
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

/// calculate insolation – the amount of radiation reaching the surface over a single watch
pub fn insolation_watch_calculate(resolution: usize, watch: usize) -> Brane<f64> {
    info!("calculating insolation map at watch {}", watch);

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| insolation_watch_calculate_point(&point, watch))
            .collect::<Vec<f64>>(),
    );
    brane.variable = format!("insolation-{}", watch);
    brane
}

/* # heat */

/* ## absorbtion */

fn heat_absorbtion_calculate_point(
    point: &Coordinate<f64>,
    insolation: &Brane<f64>,
    albedo: &Brane<f64>,
) -> f64 {
    // this should include heat capacitance (ciepło właściwe)
    insolation.get(&point) * (1.0 - albedo.get(&point))
}

/// calculate the amount of heat absorbed by the surface
pub fn heat_absorbtion_calculate(
    resolution: usize,
    insolation: &Brane<f64>,
    albedo: &Brane<f64>,
) -> Brane<f64> {
    info!("calculating heat absorbtion");

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| heat_absorbtion_calculate_point(&point, &insolation, &albedo))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "heat-absorbed".to_string();
    brane
}

/* ## diffusion */

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Medium {
    Air,
    Ocean,
}

fn viscosity(medium: Medium) -> f64 {
    match medium {
        Medium::Air => 0.32,
        Medium::Ocean => 0.96,
    }
}

fn heat_diffusion_calculate_point(
    point: &Coordinate<f64>,
    medium: Medium,
    heat: &Brane<f64>,
    surface: &Brane<u8>,
) -> f64 {
    let current = heat.get(&point);
    if medium == Medium::Air || decode(surface.get(&point)) == Surface::Water {
        let mut ambit = heat.ambit(&point);
        // the folllowing line assumes that heat and surface have the same resolution
        if medium == Medium::Ocean {
            ambit = ambit
                .into_iter()
                .filter(|gon| decode(surface.get(&gon)) == Surface::Water)
                .collect();
        }
        let len = ambit.len() as f64;
        if len > 0.0 {
            let surround = ambit.into_iter().map(|gon| heat.get(&gon)).sum::<f64>() / len;
            current + viscosity(medium) * (surround - current)
        } else {
            current
        }
    } else {
        current
    }
}

/// calculate heat diffusion through a medium
pub fn heat_diffusion_calculate(
    resolution: usize,
    medium: Medium,
    absorbtion: &Brane<f64>,
    surface: &Brane<u8>,
) -> Brane<f64> {
    info!("calculating heat diffusion through {:?}", medium);

    let mut brane = absorbtion.clone();
    for _ in 0..2 * resolution {
        // this could probalby be sped up, if was done outside of a brane environment, on pure Vecs
        brane = Brane::from(
            Brane::<f64>::vec_par_iter(resolution)
                .map(|point| heat_diffusion_calculate_point(&point, medium, &brane, &surface))
                .collect::<Vec<f64>>(),
        );
    }
    brane.variable = "heat-diffused".to_string();
    brane
}

/// calculate heat diffusion through all mediums
pub fn heat_diffusion_calculate_all(
    resolution: usize,
    absorbtion: &Brane<f64>,
    surface: &Brane<u8>,
) -> Brane<f64> {
    info!("calculating heat diffusion");

    let brane = heat_diffusion_calculate(resolution, Medium::Ocean, &absorbtion, &surface);
    heat_diffusion_calculate(resolution, Medium::Air, &brane, &surface)
}
