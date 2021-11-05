use crate::{
    carto::{
        brane::Brane,
        datum::DatumZa,
        flux::Flux,
        honeycomb::{ball_cone_volume, HoneyCellToroidal},
    },
    climate::vegetation::Vege,
    vars::*,
};
use log::trace;
use ord_subset::OrdSubsetIterExt;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;

/* # rainfall */

/* ## evaporation */

/// potential amount of water that could be evaporated
pub fn evaporation(temperature: &Brane<f64>) -> Brane<f64> {
    trace!("calculating evaporation model");
    Brane::from(
        (0..temperature.resolution.pow(2))
            .into_par_iter()
            .map(|j| temperature.grid[j].powi(2))
            .collect::<Vec<f64>>(),
    )
}

/// actual amount of water released into the atmosphere
pub fn evapotranspiration(
    evaporation: &Brane<f64>,
    vegetation: &Brane<Option<Vege>>,
) -> Brane<f64> {
    trace!("calculating evapotranspiration model");
    Brane::from(
        (0..evaporation.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                evaporation.grid[j]
                    * match &vegetation.grid[j] {
                        Some(vege) => vege.water(),
                        None => 0.84,
                    }
            })
            .collect::<Vec<f64>>(),
    )
}

/* ## rainfall */

fn rainfall_nd(
    target_level: f64,
    node: NodeIndex,
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    gradient: &Graph<DatumZa, f64>,
    rainfall: &mut Brane<f64>,
) -> f64 {
    let datum = &gradient[node];
    let index = datum.unravel(elevation.resolution);
    let level = elevation.grid[index];
    let moisture = evaporation.grid[index]
        + gradient
            .edges_directed(node, Direction::Incoming)
            .map(|edge| {
                rainfall_nd(
                    level,
                    edge.source(),
                    elevation,
                    evaporation,
                    gradient,
                    rainfall,
                )
            })
            .sum::<f64>();
    let frac = moisture
        * *[
            1.0,
            *[(target_level - level) * 8.0, FLAT_RAIN]
                .iter()
                .ord_subset_max()
                .unwrap(),
        ]
        .iter()
        .ord_subset_min()
        .unwrap();

    let radius = elevation.resolution.div_euclid(18) as i32;
    let volume = ball_cone_volume(radius) as f64;
    for nbr in datum.ball_toroidal(radius, elevation.resolution as i32) {
        rainfall.grid[nbr.unravel(elevation.resolution)] += (frac
            * (radius - datum.dist_toroidal(&nbr, elevation.resolution as i32) + 1) as f64
            * volume.recip())
        .powf(0.94);
    }
    moisture - frac
}

/// calculate the amount of rainfall reaching the surface
pub fn rainfall(elevation: &Brane<f64>, evaporation: &Brane<f64>, wind: &Flux<f64>) -> Brane<f64> {
    trace!("calculating rainfall");
    // this is slightly slower than diffusion, although it does look better

    let mut rainfall = Brane::<f64>::zeros(evaporation.resolution);
    for node in &wind.roots {
        rainfall_nd(
            0.0,
            *node,
            elevation,
            evaporation,
            &wind.graph,
            &mut rainfall,
        );
    }
    rainfall.variable = "rainfall".to_string();
    rainfall
}

/* # watershed */

/*
fn shed_nd(
    node: NodeIndex,
    rainfall: &Brane<f64>,
    elevation_flux: &Flux<f64>,
    shed: &mut Brane<f64>,
) -> f64 {
    let datum = &elevation_flux.graph[node];
    let moisture = rainfall.get(&datum.cast(elevation_flux.resolution))
        + elevation_flux
            .graph
            .edges_directed(node, Direction::Incoming)
            .map(|edge| shed_nd(edge.source(), rainfall, elevation_flux, shed))
            .sum::<f64>();
    shed.insert(&datum, moisture);
    moisture
}

/// calculate the amount of water flowing down to every datum
pub fn shed(elevation_flux: &Flux<f64>, rainfall: &Brane<f64>) -> Brane<f64> {
    trace!("calculating watershed");

    let mut shed = Brane::<f64>::zeros(elevation_flux.resolution);
    for node in &elevation_flux.roots {
        shed_nd(*node, rainfall, elevation_flux, &mut shed);
    }

    shed.variable = "shed".to_string();
    shed
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn evaporation_values() {
        let brane = evaporation(&Brane::from(
            (0..36).map(|j| j as f64 / 36.0).collect::<Vec<f64>>(),
        ));
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.049382, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.444444, abs <= EPSILON);
    }

    #[test]
    fn evapotraspiration_values() {
        let brane = evapotranspiration(
            &Brane::from((0..4).map(|j| j as f64 / 3.0).collect::<Vec<f64>>()),
            &Brane::from(vec![
                Some(Vege::Broadleaf),
                None,
                Some(Vege::Broadleaf),
                Some(Vege::Prairie),
            ]),
        );
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[1], 0.279999, abs <= EPSILON);
        assert_float_eq!(brane.grid[2], 0.666666, abs <= EPSILON);
        assert_float_eq!(brane.grid[3], 0.18, abs <= EPSILON);
    }

    #[test]
    fn rainfall_values() {
        let brane = rainfall(
            &Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>()),
            &Brane::from((0..36).map(|j| (j % 3) as f64).collect::<Vec<f64>>()),
            &Flux::<f64>::from(&Brane::from(
                (0..36).map(|j| j as f64).collect::<Vec<f64>>(),
            )),
        );
        assert_float_eq!(brane.grid[0], 3.198547, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.089760, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.0, abs <= EPSILON);
    }

    /*
    #[test]
    fn shed_values() {
        let brane = shed(
            &Flux::<f64>::from(&Brane::from(
                (0..36).map(|j| j as f64).collect::<Vec<f64>>(),
            )),
            &Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>()),
        );
        assert_float_eq!(brane.grid[0], 630.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 68.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 24.0, abs <= EPSILON);
    }
    */
}
