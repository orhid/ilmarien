use crate::climate::surface::{decode, Surface};
use crate::imaging::{
    cartography::{new, Brane},
    hexagonos::{Gon, PreGon},
};
use geo_types::Coordinate;
//use log::debug;
use log::info;
use nalgebra::Vector3;
use rayon::prelude::*;
use std::f64::consts::TAU;

/* insolation */

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

/* +daily */

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

pub fn insolation_calculate(resolution: usize) -> Brane {
    //! calculate insolation – the amount of radiation reaching the surface over a single day

    info!("calculating insolation map");

    let mut brane = new("insolation".to_string(), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| insolation_calculate_point(&point))
            .collect(),
    );
    brane
}

/* +watchly */

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

pub fn insolation_watch_calculate(resolution: usize, watch: usize) -> Brane {
    //! calculate insolation – the amount of radiation reaching the surface over a single watch

    info!("calculating insolation map at watch {}", watch);

    let mut brane = new(format!("insolation-{}", watch), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| insolation_watch_calculate_point(&point, watch))
            .collect(),
    );
    brane
}

/* heat */

/* +absorbtion */

fn heat_absorbtion_calculate_point(
    point: &Coordinate<f64>,
    insolation: &Brane,
    albedo: &Brane,
) -> f64 {
    // this should include heat capacitance (ciepło właściwe)
    insolation.find_value(&point) * (1.0 - albedo.find_value(&point))
}

pub fn heat_absorbtion_calculate(resolution: usize, insolation: &Brane, albedo: &Brane) -> Brane {
    //! calculate the amount of heat absorbed by the surface

    info!("calculating heat absorbtion");

    let mut brane = new("heat-absorbed".to_string(), resolution);
    brane.engrid(
        brane
            .into_par_iter()
            .map(|point| heat_absorbtion_calculate_point(&point, &insolation, &albedo))
            .collect(),
    );
    brane
}

/* *diffusion */

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
    heat: &Brane,
    surface: &Brane,
) -> f64 {
    let current = heat.find_value(&point);
    if medium == Medium::Air || decode(surface.find_value_exact(&point)) == Surface::Water {
        let mut ambit = heat.find(&point).ambit(heat.resolution as i32);
        // the folllowing line assumes that heat and surface have the same resolution
        if medium == Medium::Ocean {
            ambit = ambit
                .into_iter()
                .filter(|gon| decode(surface.get_exact(&gon)) == Surface::Water)
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

pub fn heat_diffusion_calculate(
    resolution: usize,
    medium: Medium,
    absorbtion: &Brane,
    surface: &Brane,
) -> Brane {
    //! calculate heat diffusion through a medium

    info!("calculating heat diffusion through {:?}", medium);

    let mut brane = new("heat-diffused".to_string(), resolution);
    brane.engrid(
        absorbtion
            .into_iter()
            .map(|point| absorbtion.get(&point))
            .collect(),
    );
    for _ in 0..2 * resolution {
        // this could probalby be sped up, if was done outside of a brane environment, on pure Vecs
        brane.engrid(
            brane
                .into_par_iter()
                .map(|point| heat_diffusion_calculate_point(&point, medium, &brane, &surface))
                .collect(),
        );
    }
    brane
}

pub fn heat_diffusion_calculate_all(
    resolution: usize,
    absorbtion: &Brane,
    surface: &Brane,
) -> Brane {
    //! calculate heat diffusion through all mediums

    info!("calculating heat diffusion");

    let brane = heat_diffusion_calculate(resolution, Medium::Ocean, &absorbtion, &surface);
    heat_diffusion_calculate(resolution, Medium::Air, &brane, &surface)
}
