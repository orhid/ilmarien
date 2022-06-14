use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::HoneyCellToroidal,
    },
    climate::{hydrology::shed, vegetation::Vege},
    units::{Elevation, Unit},
};
use log::trace;
use noise::{NoiseFn, OpenSimplex, Seedable};
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::collections::VecDeque;
use std::f64::consts::TAU;

const SQRT3B2: f64 = 0.8660254;

/* # bedrock generation */

fn bedrock_elevation_at_datum(
    datum: &DatumRe,
    noise: &OpenSimplex,
    curve: &Spline<f64, f64>,
) -> Elevation {
    let fractional_brownian_motion = |x: f64, y: f64| {
        (0..8)
            .map(|level| {
                let freq = 2f64.powi(level - 1);
                1.8f64.powi(-level)
                    * noise.get([
                        // toroidal wrapping
                        freq * x.cos(),
                        freq * x.sin(),
                        freq * SQRT3B2 * y.cos(), // undistort geometry on the hexagon
                        freq * SQRT3B2 * y.sin(), // undistort geometry on the hexagon
                    ])
            })
            .sum::<f64>()
    };

    let x: f64 = TAU * datum.x;
    let y: f64 = TAU * (datum.x + datum.y); // undistort geometry on the hexagon
    let toroidal_sample = {
        fractional_brownian_motion(
            x + fractional_brownian_motion(x, y),
            y + fractional_brownian_motion(x, y),
        )
    };

    Elevation::confine(curve.clamped_sample(0.84 * toroidal_sample).unwrap())
}

/// generate a bedrock elevation model from noise
pub fn bedrock_elevation(resolution: usize, seed: u32) -> Brane<Elevation> {
    trace!("generating bedrock elevation model");

    let noise = OpenSimplex::new().set_seed(seed);
    let elevation_curve: Spline<f64, f64> = {
        // curve moves the mode of the distribution
        let step = 256f64.recip();
        let shelf: f64 = 0.27;
        Spline::from_vec(vec![
            Key::new(-1., 0., Interpolation::Linear),
            Key::new(-0.72, shelf - 21.0 * step, Interpolation::Linear),
            Key::new(-0.15, shelf - 2.0 * step, Interpolation::Linear),
            Key::new(0.0, shelf, Interpolation::Linear),
            Key::new(0.03, shelf + 4.0 * step, Interpolation::Linear),
            Key::new(0.27, shelf + 16.0 * step, Interpolation::Linear),
            Key::new(1., 1., Interpolation::Linear),
        ])
    };

    Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                bedrock_elevation_at_datum(
                    &DatumZa::enravel(j, resolution).cast(resolution),
                    &noise,
                    &elevation_curve,
                )
            })
            .collect::<Vec<Elevation>>(),
    )
}

pub fn ocean_level(elevation: &Brane<Elevation>) -> Elevation {
    let mut v = elevation
        .grid
        .iter()
        .map(|j| j.release())
        .collect::<Vec<f64>>();
    v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    Elevation::confine(
        v.group_by(|a, b| ((a - b) / a).abs() < 0.000_01)
            .max_by(|a, b| a.len().cmp(&b.len()))
            .expect("every group will have at least one element")
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - 256f64.recip(),
    )
}

/* # ... */

pub fn bedrock_vege(elevation: &Brane<f64>, ocean: f64) -> Brane<Option<Vege>> {
    Brane::from(
        (0..elevation.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                if elevation.grid[j] < ocean {
                    None
                } else {
                    Some(Vege::Stone)
                }
            })
            .collect::<Vec<Option<Vege>>>(),
    )
}

/* # continentality */

pub fn elevation_above_ocean(elevation: &Brane<Elevation>, ocean: Elevation) -> Brane<Elevation> {
    Brane::from(
        (0..elevation.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                let elevation_here = elevation.grid[j];
                if elevation_here > ocean {
                    elevation_here
                } else {
                    ocean
                }
            })
            .collect::<Vec<Elevation>>(),
    )
}

/// find distance to closest ocean, going around mountains
pub fn continentality(elevation: &Brane<Elevation>, ocean: Elevation) -> Brane<f64> {
    trace!("calculating continentality coefficients");

    let resolution = elevation.resolution;
    let mountains_elevation: i32 = 1728;

    // find mountain tiles
    let mountain_tiles: Brane<bool> = Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| elevation.grid[j].meters() > ocean.meters() + mountains_elevation)
            .collect::<Vec<bool>>(),
    );

    // find ocean tiles
    let ocean_tiles: Brane<bool> = Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| elevation.grid[j] < ocean)
            .collect::<Vec<bool>>(),
    );

    let mut continentality = Brane::from(
        mountain_tiles
            .grid
            .par_iter()
            .map(|b| if *b { Some(-1) } else { None })
            .collect::<Vec<Option<i32>>>(),
    );

    // poulate oceans with zeros
    let mut ocean_datums = VecDeque::new();
    for index in 0..resolution.pow(2) {
        if ocean_tiles.grid[index] {
            continentality.grid[index] = Some(0);
            ocean_datums.push_back(DatumZa::enravel(index, resolution));
        }
    }

    // flood fill from ocean datums
    while !ocean_datums.is_empty() {
        let here = ocean_datums.pop_front().unwrap();
        for datum in here.ambit_toroidal(resolution as i32) {
            let index = datum.unravel(resolution);
            if continentality.grid[index].is_none() {
                continentality.grid[index] = Some(
                    continentality.grid[here.unravel(resolution)]
                        .expect("has already been assigned")
                        + 1,
                );
                ocean_datums.push_back(datum);
            }
        }
    }

    Brane::from(
        continentality
            .grid
            .into_par_iter()
            .map(|j| match j {
                Some(v) => match v {
                    -1 => 1.0,
                    x => 12. * (x as f64) / (resolution as f64),
                },
                None => 1.0,
            })
            .collect::<Vec<f64>>(),
    )
}

/* # erosion */

const MAX_RAIN: f64 = 24.0;
const MAX_DELTA: f64 = 0.1296;
const BOUNCEBACK: f64 = 0.012;

fn erode_nd(
    node: NodeIndex,
    elevation: &mut Brane<f64>,
    shed: &Brane<f64>,
    gradient: &Graph<DatumZa, f64>,
) {
    let alt_here = elevation.read(&gradient[node]);
    for edge in gradient.edges_directed(node, Direction::Incoming) {
        let source = edge.source();
        let child = gradient[source];
        let rain = shed.read(&child);
        if rain > 1.0 {
            elevation.grid[child.unravel(elevation.resolution)] = alt_here
                + MAX_DELTA.min((elevation.read(&child) - alt_here) * (MAX_RAIN.min(rain)).recip())
                + BOUNCEBACK;
        } else {
            // thermal erosion, could for example tend towards the average of its neighbours if it is
            // above sea level
        }
        erode_nd(source, elevation, shed, gradient);
    }
}

const EROSION_LOOP: usize = 1;

pub fn erode(elevation: &mut Brane<f64>, rain: &Brane<f64>) {
    trace!("calculating erosion");
    for _ in 0..EROSION_LOOP {
        let slope = Flux::<f64>::from(&elevation.clone());
        let shed = shed(&slope, rain);
        for node in &slope.roots {
            erode_nd(*node, elevation, &shed, &slope.graph);
            elevation.grid[slope.graph[*node].unravel(elevation.resolution)] += BOUNCEBACK;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn bedrock_elevation_values() {
        let brane = bedrock_elevation(6, 0);
        assert_float_eq!(brane.grid[0].release(), 0.264407, abs <= EPSILON);
        assert_float_eq!(brane.grid[8].release(), 0.273077, abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.285910, abs <= EPSILON);

        let brane = bedrock_elevation(6, 1);
        assert_float_ne!(brane.grid[0].release(), 0.264407, abs <= EPSILON);
        assert_float_ne!(brane.grid[8].release(), 0.273077, abs <= EPSILON);
        assert_float_ne!(brane.grid[24].release(), 0.285910, abs <= EPSILON);
    }

    #[test]
    fn bedrock_elevation_tileability() {
        let noise = OpenSimplex::new();
        let curve: Spline<f64, f64> = Spline::from_vec(vec![
            Key::new(0., 0., Interpolation::Linear),
            Key::new(1., 1., Interpolation::Linear),
        ]);
        assert_float_eq!(
            bedrock_elevation_at_datum(&DatumRe::new(0.0, 0.1), &noise, &curve).release(),
            bedrock_elevation_at_datum(&DatumRe::new(0.0, 1.1), &noise, &curve).release(),
            abs <= EPSILON,
        );
        assert_float_eq!(
            bedrock_elevation_at_datum(&DatumRe::new(0.1, 0.0), &noise, &curve).release(),
            bedrock_elevation_at_datum(&DatumRe::new(1.1, 0.0), &noise, &curve).release(),
            abs <= EPSILON,
        );
    }

    #[test]
    fn continentality_values() {
        let brane = continentality(
            &Brane::from(
                (0..36)
                    .map(|j| Elevation::new((j % 6) as f64))
                    .collect::<Vec<Elevation>>(),
            ),
            Elevation::new(3_f64.recip()),
        );
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[5], 1.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[6], 0.0, abs <= EPSILON);
    }
}
