use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        flux::Flux,
    },
    climate::cosmos::Fabric,
    util::diffusion::diffuse_level,
    vars::*,
};
use log::{trace, warn};
use ord_subset::OrdSubsetIterExt;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use rayon::prelude::*;

/* # rainfall */

/* ## evaporation */

fn evaporation_rate(pressure: f64, temperature: f64) -> f64 {
    (temperature * pressure.recip()) * 0.00288
}

fn evaporation_dt(
    datum: &DatumRe,
    pressure: &Brane<f64>,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
) -> f64 {
    let rate = evaporation_rate(pressure.get(&datum), temperature.get(&datum));
    match surface.get(&datum) {
        Fabric::Water => rate,
        Fabric::Ice | Fabric::Snow => 0.12 * rate,
        _ => 0.0,
    }
}

/// calculate evaporation rate
pub fn evaporation(
    pressure: &Brane<f64>,
    surface: &Brane<Fabric>,
    temperature: &Brane<f64>,
) -> Brane<f64> {
    trace!("calculating evaporation rate");

    if temperature.resolution != pressure.resolution {
        warn!("evaporation: branes at incompatible resolutions: temperature, pressure");
    }

    let mut brane = Brane::from(
        (0..temperature.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                evaporation_dt(
                    &DatumZa::enravel(j, temperature.resolution).cast(temperature.resolution),
                    &pressure,
                    &surface,
                    &temperature,
                )
            })
            .collect::<Vec<f64>>(),
    );
    brane.variable = "evaporation".to_string();
    brane
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
    let level = elevation.get(&datum.cast(evaporation.resolution));
    let moisture = evaporation.read(&datum)
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
    rainfall.insert(&datum, frac);
    moisture - frac
}

/// calculate the amount of rainfall reaching the surface
pub fn rainfall(elevation: &Brane<f64>, evaporation: &Brane<f64>, wind: &Flux<f64>) -> Brane<f64> {
    trace!("calculating rainfall");

    if evaporation.resolution != wind.resolution {
        warn!("rainfall: branes at incompatible resolutions: evaporation, pressure");
    }

    let mut rainfall = Brane::<f64>::zeros(evaporation.resolution);
    let residues = wind
        .roots
        .iter()
        .map(|node| {
            rainfall_nd(
                0.0,
                *node,
                elevation,
                evaporation,
                &wind.graph,
                &mut rainfall,
            )
        })
        .collect::<Vec<f64>>();

    for (j, node) in wind.roots.iter().enumerate() {
        rainfall.grid[wind.graph[*node].unravel(rainfall.resolution)] += residues[j];
    }

    for _ in 0..rainfall.resolution.pow(2) / 324 {
        rainfall.grid = (0..rainfall.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                diffuse_level(
                    &DatumZa::enravel(j, rainfall.resolution).cast(rainfall.resolution),
                    &rainfall,
                    &elevation,
                )
            })
            .collect::<Vec<f64>>();
    }
    rainfall.variable = "rainfall".to_string();
    rainfall
}

/* # watershed */

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

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn evaporation_values() {
        let brane = evaporation(
            &Brane::from((0..36).map(|_| 1f64).collect::<Vec<f64>>()),
            &Brane::from((0..36).map(|_| Fabric::Water).collect::<Vec<Fabric>>()),
            &Brane::from((0..36).map(|j| j as f64 + 273.0).collect::<Vec<f64>>()),
        );
        assert_float_eq!(brane.grid[0], 0.78624, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 0.80928, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.85536, abs <= EPSILON);
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
        assert_float_eq!(brane.grid[0], 19.138650, abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.095756, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0.0, abs <= EPSILON);
    }

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

    #[test]
    fn hydrolic_cylce_closure() {
        // check whether the sum of water at shed roots is equal to the sum of water evaporating
        let evaporation = Brane::from((0..36).map(|_| 1f64).collect::<Vec<f64>>());
        let flux = Flux::<f64>::from(&Brane::from(
            (0..36).map(|j| j as f64).collect::<Vec<f64>>(),
        ));
        let shed = shed(
            &flux,
            &rainfall(
                &Brane::from((0..36).map(|j| j as f64).collect::<Vec<f64>>()),
                &evaporation,
                &Flux::<f64>::from(&Brane::from(
                    (0..36).map(|j| j as f64).collect::<Vec<f64>>(),
                )),
            ),
        );
        assert_float_eq!(
            flux.roots
                .iter()
                .map(|node| shed.read(&flux.graph[*node]))
                .sum::<f64>(),
            evaporation.grid.iter().sum::<f64>(),
            abs <= EPSILON
        )
    }
}
