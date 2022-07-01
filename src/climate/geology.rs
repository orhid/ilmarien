use crate::{
    carto::{
        brane::{Brane, Resolution},
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::HoneyCellToroidal,
    },
    climate::hydrology::shed,
    units::{Elevation, Precipitation, Unit},
};
use log::trace;
use noise::{NoiseFn, OpenSimplex, Seedable};
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use splines::{Interpolation, Key, Spline};
use std::f64::consts::TAU;

const SQRT3B2: f64 = 0.8660254;

/* # bedrock generation */

/// generate a bedrock elevation model from noise
pub fn bedrock_elevation(resolution: Resolution, seed: u32) -> Brane<Elevation> {
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

    let toroidal_sample = |datum: &DatumRe| -> f64 {
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
        fractional_brownian_motion(
            x + fractional_brownian_motion(x, y),
            y + fractional_brownian_motion(x, y),
        )
    };

    Brane::<Elevation>::create_by_index(resolution, |j| {
        Elevation::confine(
            elevation_curve
                .clamped_sample(
                    0.84 * toroidal_sample(&DatumZa::enravel(j, resolution).cast(resolution)),
                )
                .unwrap(),
        )
    })
}

pub fn ocean_level(_elevation: &Brane<Elevation>) -> Elevation {
    // this is an average from some simulations
    Elevation::confine(0.2625598904721348)
    /*
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
    */
}

pub fn ocean_tiles(elevation: &Brane<Elevation>, ocean: Elevation) -> Brane<bool> {
    elevation.operate_by_value_ref(|value| value < &ocean)
}

pub fn altitude_above_ocean_level(
    elevation: &Brane<Elevation>,
    ocean: Elevation,
) -> Brane<Elevation> {
    elevation.operate_by_value_ref(|value| {
        Elevation::confine((value.release() - ocean.release()).max(0.))
    })
}

/* # erosion */

const MAX_DELTA: f64 = 0.1296;

pub fn erode(
    elevation: &mut Brane<Elevation>,
    rain: &Brane<Precipitation>,
    mountain_level: Elevation,
) {
    trace!("calculating erosion");

    fn erode_at_node(
        node: NodeIndex,
        elevation: &mut Brane<Elevation>,
        shed: &Brane<Precipitation>,
        slope: &Graph<DatumZa, Elevation>,
        mountain_level: Elevation,
    ) {
        let elevation_here = elevation.grid[slope[node].unravel(shed.resolution)];
        for edge in slope.edges_directed(node, Direction::Incoming) {
            let source = edge.source();
            let child = slope[source];
            let avg = child
                .ambit_toroidal(elevation.resolution.into())
                .map(|neighbour| elevation.grid[neighbour.unravel(elevation.resolution)].release())
                .into_iter()
                .sum::<f64>()
                * 6f64.recip();
            if let Some(elevation_source) =
                elevation.grid.get_mut(child.unravel(elevation.resolution))
            {
                if *elevation_source > mountain_level {
                    *elevation_source = elevation_here
                        + Elevation::confine(
                            MAX_DELTA.min((*elevation_source - elevation_here).release())
                                * (1.
                                    - 1f64.min(
                                        shed.grid[child.unravel(shed.resolution)]
                                            .release()
                                            .powf(3f64.recip()), // the root forces more erosion in arid regions
                                    )),
                        );
                    // + Elevation::confine(BOUNCEBACK);
                } else {
                    let k = 0.7;
                    *elevation_source =
                        Elevation::confine(k * elevation_source.release() + (1. - k) * avg);
                }
            }
            erode_at_node(source, elevation, shed, slope, mountain_level);
        }
    }

    for _ in 0..1 {
        let slope = Flux::<Elevation>::from(elevation.clone());
        let shed = shed(&slope, rain);
        for node in slope.roots.clone() {
            erode_at_node(node, elevation, &shed, &slope.graph, mountain_level);
            /*
            let elevation_here = elevation
                .grid
                .get_mut(slope.graph[*node].unravel(elevation.resolution))
                .expect("index out of bouds");
            *elevation_here = *elevation_here + Elevation::confine(BOUNCEBACK);
            */
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;
    const RES: Resolution = Resolution::confine(6);

    #[test]
    fn bedrock_elevation_values() {
        let brane = bedrock_elevation(RES, 0);
        assert_float_eq!(brane.grid[0].release(), 0.264407, abs <= EPSILON);
        assert_float_eq!(brane.grid[8].release(), 0.273077, abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.285910, abs <= EPSILON);

        let brane = bedrock_elevation(RES, 1);
        assert_float_ne!(brane.grid[0].release(), 0.264407, abs <= EPSILON);
        assert_float_ne!(brane.grid[8].release(), 0.273077, abs <= EPSILON);
        assert_float_ne!(brane.grid[24].release(), 0.285910, abs <= EPSILON);
    }
}
