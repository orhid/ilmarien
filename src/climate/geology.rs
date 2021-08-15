use crate::imaging::cartography::Brane;
use crate::util::consants::*;
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
    let x: f64 = TAU * point.x;
    let y: f64 = TAU * point.y;

    let amplitude: f64 = (0..GEO_DETAIL)
        .map(|amp| AMP_FACTOR.powi(-amp))
        .sum::<f64>();

    let value = (0..GEO_DETAIL)
        .map(|level| {
            let freq = GEO_SCALE * 2.0_f64.powi(level);
            AMP_FACTOR.powi(-level)
                * noise.get([
                    freq * x.cos(),
                    freq * x.sin(),
                    freq * DST_FACTOR * y.cos(),
                    freq * DST_FACTOR * y.sin(),
                ])
        })
        .sum::<f64>();
    curve
        .clamped_sample(BLW_FACTOR * value / amplitude)
        .unwrap()
}

/// generate an elevation model from Perlin noise
/// the values will range from 0.0 to 1.0
/// they correspond to the difference between 0 and 13824 meters
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
