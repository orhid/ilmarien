use crate::imaging::cartography::Brane;
use geo_types::Coordinate;
use log::info;
use noise::{NoiseFn, OpenSimplex, Seedable};
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::f64::consts::TAU;

fn elevation_ease_curve() -> Spline<f64, f64> {
    Spline::from_vec(vec![
        Key::new(-1., 0., Interpolation::Linear),
        Key::new(-0.4744, 0.1843, Interpolation::Linear),
        Key::new(-0.2806, 0.2157, Interpolation::Linear),
        Key::new(-0.1693, 0.2392, Interpolation::Linear),
        Key::new(-0.1094, 0.2471, Interpolation::Linear),
        Key::new(-0.0416, 0.2549, Interpolation::Linear),
        Key::new(0.0367, 0.2627, Interpolation::Linear),
        Key::new(0.1326, 0.2784, Interpolation::Linear),
        Key::new(0.2353, 0.3098, Interpolation::Linear),
        Key::new(0.3628, 0.3725, Interpolation::Linear),
        Key::new(1., 1., Interpolation::Linear),
    ])
}

fn elevation_generate_point(
    point: &Coordinate<f64>,
    noise: &OpenSimplex,
    curve: &Spline<f64, f64>,
) -> f64 {
    let detail: i32 = 12;
    let x: f64 = TAU * point.x;
    let y: f64 = TAU * point.y;

    let amplifactor: f64 = 1.44;
    let amplitude: f64 = (0..detail)
        .map(|ampli| amplifactor.powi(-ampli))
        .sum::<f64>();

    let value = (0..detail)
        .map(|level| {
            let freq = 0.84 * 2.0_f64.powi(level); // the first number controls scale
            let ampli = amplifactor.powi(-level);
            let factor: f64 = 3.0_f64.sqrt() / 2.0; //this should lessen the distortion

            ampli
                * noise.get([
                    freq * x.cos(),
                    freq * x.sin(),
                    freq * factor * y.cos(),
                    freq * factor * y.sin(),
                ])
        })
        .sum::<f64>();
    curve.clamped_sample(1.68 * value / amplitude).unwrap()
}

/// generate an elevation model from Perlin noise
pub fn elevation_generate(resolution: usize, seed: u32) -> Brane<f64> {
    info!("generating elevation model");
    let noise = OpenSimplex::new().set_seed(seed);
    let curve = elevation_ease_curve();

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| elevation_generate_point(&point, &noise, &curve))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "elevation".to_string();
    brane
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn elevation_generate_tileability() {
        let noise = OpenSimplex::new();
        let curve = elevation_ease_curve();
        assert_float_eq!(
            elevation_generate_point(&Coordinate { x: 0.0, y: 0.1 }, &noise, &curve),
            elevation_generate_point(&Coordinate { x: 0.0, y: 1.1 }, &noise, &curve),
            abs <= EPSILON,
        );
        assert_float_eq!(
            elevation_generate_point(&Coordinate { x: 0.1, y: 0.0 }, &noise, &curve),
            elevation_generate_point(&Coordinate { x: 1.1, y: 0.0 }, &noise, &curve),
            abs <= EPSILON,
        );
    }
}
