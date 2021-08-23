use crate::{
    carto::{brane::Brane, datum::DatumRe},
    util::constants::*,
};
use log::info;
use noise::{NoiseFn, OpenSimplex, Seedable};
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::f64::consts::TAU;

/* # bedrock generation */

fn elevation_curve() -> Spline<f64, f64> {
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

fn bedrock_level_dt(datum: &DatumRe, noise: &OpenSimplex, curve: &Spline<f64, f64>) -> f64 {
    let x: f64 = TAU * datum.x;
    let y: f64 = TAU * datum.y;

    let value = (0..GEO_DETAIL)
        .map(|level| {
            let freq = GEO_SCALE * 2.0f64.powi(level);
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
        .clamped_sample(BLW_FACTOR * value * (1.0 - AMP_FACTOR.recip()))
        .unwrap()
}

/**
 * generate a bedrock elevation model from Perlin noise
 * the values will range from 0.0 to 1.0
 * they correspond to the difference between 0 and 13824 meters
 */
pub fn bedrock_level(resolution: usize, seed: u32) -> Brane<f64> {
    info!("generating bedrock levels");
    let noise = OpenSimplex::new().set_seed(seed);
    let curve = elevation_curve();

    let mut brane = Brane::from(
        Brane::<f64>::par_iter(resolution)
            .map(|datum| bedrock_level_dt(&datum, &noise, &curve))
            .collect::<Vec<f64>>(),
    );
    brane.variable = "bedrock".to_string();
    brane
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn bedrock_level_values() {
        let brane = bedrock_level(6, 0);
        assert_float_eq!(brane.grid[0], 0.262396, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.295638, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.247292, abs <= EPSILON);

        let brane = bedrock_level(6, 1);
        assert_float_ne!(brane.grid[0], 0.262396, abs <= EPSILON);
        assert_float_ne!(brane.grid[8], 0.295638, abs <= EPSILON);
        assert_float_ne!(brane.grid[24], 0.247292, abs <= EPSILON);
    }

    #[test]
    fn bedrock_level_tileability() {
        let noise = OpenSimplex::new();
        let curve = elevation_curve();
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.0, 0.1), &noise, &curve),
            bedrock_level_dt(&DatumRe::new(0.0, 1.1), &noise, &curve),
            abs <= EPSILON,
        );
        assert_float_eq!(
            bedrock_level_dt(&DatumRe::new(0.1, 0.0), &noise, &curve),
            bedrock_level_dt(&DatumRe::new(1.1, 0.0), &noise, &curve),
            abs <= EPSILON,
        );
    }
}
