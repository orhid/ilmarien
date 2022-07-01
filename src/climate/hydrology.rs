use crate::{
    carto::{brane::Brane, datum::DatumZa, flux::Flux, honeycomb::HoneyCellToroidal},
    units::{Elevation, Precipitation, Temperature, Unit},
};
use log::trace;
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};
use std::collections::{HashMap, VecDeque};

/* # continentality */

fn cowind(
    elevation_source: Elevation,
    temperature_source: Temperature,
    elevation_target: Elevation,
    temperature_target: Temperature,
) -> f64 {
    1. + (temperature_target.release() - temperature_source.release())
        + 2f64.powf((elevation_target.release() - elevation_source.release()).max(0.))
}

pub fn continentality(
    altitude_above_ocean: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    ocean_tiles: &Brane<bool>,
) -> Brane<f64> {
    trace!("calculating continentality");
    let resolution = altitude_above_ocean.resolution;

    // create storage and prepopulate with infinities
    let mut continentality = Brane::new(vec![f64::INFINITY; resolution.square()], resolution);
    let mut queue = VecDeque::new();

    // preopulate oceans with zeros
    for jndex in 0..resolution.square() {
        if ocean_tiles.grid[jndex] {
            continentality.grid[jndex] = 0.;
            queue.push_back(DatumZa::enravel(jndex, resolution));
        }
    }

    while !queue.is_empty() {
        let here = queue.pop_front().expect("queue was not empty");
        let here_jndex = here.unravel(resolution);
        for neighbour in here.ambit_toroidal(resolution.into()) {
            let jndex = neighbour.unravel(resolution);
            let total_cowind = continentality.grid[here_jndex]
                + cowind(
                    altitude_above_ocean.grid[here_jndex],
                    temperature.grid[here_jndex],
                    altitude_above_ocean.grid[jndex],
                    temperature.grid[jndex],
                ) / f64::from(resolution);
            if continentality.grid[jndex] > total_cowind {
                continentality.grid[jndex] = total_cowind;
                queue.push_back(neighbour);
            }
        }
    }

    continentality
}

/* # evaporation */

/// potential amount of water that could be evaporated
pub fn evapotranspiration_potential(temperature: &Brane<Temperature>) -> Brane<Precipitation> {
    temperature.operate_by_value_ref(|value| {
        Precipitation::confine(
            (value.celcius().max(0.) * Temperature::celcius_max().recip()).powi(2),
        )
    })
}

/* # rainfall */

pub fn rainfall(
    altitude_above_ocean: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    evaporation: &Brane<Precipitation>,
    ocean_tiles: &Brane<bool>,
) -> Brane<Precipitation> {
    let resolution = altitude_above_ocean.resolution;

    // create storage and prepopulate
    let mut precipitation =
        Brane::<Option<Precipitation>>::new(vec![None; resolution.square()], resolution);
    let mut queue = VecDeque::new();

    // preopulate oceans with evaporation
    for jndex in 0..resolution.square() {
        if ocean_tiles.grid[jndex] {
            precipitation.grid[jndex] = Some(evaporation.grid[jndex]);
            queue.push_back(DatumZa::enravel(jndex, resolution));
        }
    }

    let distance = |source: &DatumZa, target: &DatumZa| -> f64 {
        let mut visited = HashMap::<DatumZa, f64>::new();
        let mut unvisited = HashMap::<DatumZa, f64>::new();
        unvisited.insert(*target, 0.);
        while !unvisited.contains_key(source) {
            let here = *unvisited
                .iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .expect("there should be some unvisited nodes")
                .0;
            let here_jndex = here.unravel(resolution);
            for neighbour in here.ambit_toroidal(resolution.into()) {
                let neighbour_jndex = neighbour.unravel(resolution);
                let total_cowind = unvisited.get(&here).expect("here was unvisited")
                    + cowind(
                        altitude_above_ocean.grid[neighbour_jndex],
                        temperature.grid[neighbour_jndex],
                        altitude_above_ocean.grid[here_jndex],
                        temperature.grid[here_jndex],
                    );
                let entry = unvisited.entry(neighbour).or_insert(total_cowind);
                if *entry > total_cowind {
                    *entry = total_cowind;
                }
            }
            visited.insert(here, unvisited.remove(&here).expect("here was unvisited"));
        }
        *unvisited.get(source).expect("already found target")
    };

    while !queue.is_empty() {
        let here = queue.pop_front().expect("queue was not empty");
        for neighbour in here.ambit_toroidal(resolution.into()) {
            let jndex = neighbour.unravel(resolution);
            if precipitation.grid[jndex].is_none() {
                let mut visited = HashMap::<DatumZa, f64>::new();
                let mut candidate = (neighbour, 0.);
                while match visited.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
                    Some(pair) => *pair.1,
                    None => -1.,
                } < candidate.1
                {
                    visited.insert(candidate.0, candidate.1);
                    candidate = candidate
                        .0
                        .ambit_toroidal(resolution.into())
                        .iter()
                        .map(|child| {
                            (*child, {
                                let child_jndex = child.unravel(resolution);
                                (1. - 12f64 // higher factor reduces the distance rain can travel
                                    * distance(child, &neighbour)
                                    * f64::from(resolution).recip())
                                    * evaporation.grid[child_jndex].release().min(
                                        precipitation.grid[child_jndex]
                                            .unwrap_or_else(|| Precipitation::confine(0.))
                                            .release(),
                                    )
                            })
                        })
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        .expect("ambit is nonempty")
                }

                precipitation.grid[jndex] = Some(Precipitation::confine({
                    *visited
                        .iter()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                        .expect("something will be visited")
                        .1
                }));

                queue.push_back(neighbour);
            }
        }
    }

    precipitation.operate_by_value(Option::unwrap)
}

/* # watershed */

/// calculate the amount of water flowing down to every datum
pub fn shed(slope: &Flux<Elevation>, rainfall: &Brane<Precipitation>) -> Brane<Precipitation> {
    fn shed_at_node(
        node: NodeIndex,
        shed: &mut Brane<Precipitation>,
        slope: &Flux<Elevation>,
        rainfall: &Brane<Precipitation>,
    ) -> Precipitation {
        // this has to be a function, not a closure
        //    sincle closures cannot be called recursively
        let datum = slope.graph[node];
        let moisture = rainfall.grid[datum.unravel(slope.resolution)]
            + slope
                .graph
                .edges_directed(node, Direction::Incoming)
                .map(|edge| shed_at_node(edge.source(), shed, slope, rainfall))
                .reduce(|a, b| a + b)
                .unwrap_or(Precipitation::confine(0.));
        shed.grid[datum.unravel(slope.resolution)] = moisture;
        moisture
    }
    let mut shed = rainfall.operate_by_index(|_| Precipitation::confine(0.));
    for node in slope.roots.clone() {
        shed_at_node(node, &mut shed, slope, rainfall);
    }
    shed
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::carto::brane::Resolution;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;
    const RES: Resolution = Resolution::confine(6);
    const RES_BIG: Resolution = Resolution::confine(12);

    #[test]
    fn continentality_values() {
        let brane = continentality(
            &Brane::create_by_index(RES, |j| {
                Elevation::confine(((j % 6) as f64) - 3f64.recip().max(0.))
            }),
            &Brane::create_by_index(RES, |j| Temperature::confine((j % 6) as f64)),
            &Brane::create_by_index(RES, |j| ((j % 6) as f64) < 3f64.recip()),
        );
        assert_float_eq!(brane.grid[0], 0., abs <= EPSILON);
        assert_float_eq!(brane.grid[8], 1.264567, abs <= EPSILON);
        assert_float_eq!(brane.grid[24], 0., abs <= EPSILON);
    }

    #[test]
    fn evapotranspiration_potential_values() {
        let brane = evapotranspiration_potential(&Brane::create_by_index(RES, |j| {
            Temperature::confine(j as f64 / 36.)
        }));
        assert_float_eq!(brane.grid[0].release(), 0., abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.111111, abs <= EPSILON);
        assert_float_eq!(brane.grid[32].release(), 0.604938, abs <= EPSILON);
    }

    #[test]
    fn rainfall_values() {
        let brane = rainfall(
            &Brane::create_by_index(RES_BIG, |j| {
                Elevation::confine((((j % 12 + j / 12) as f64 / 24.) - 6f64.recip()).max(0.))
            }),
            &Brane::create_by_index(RES_BIG, |j| {
                Temperature::confine(1. - ((j % 12 + j / 12) as f64 / 24.))
            }),
            &Brane::create_by_index(RES_BIG, |j| {
                Precipitation::confine(1. - ((j % 12 + j / 12) as f64 / 24.))
            }),
            &Brane::create_by_index(RES_BIG, |j| ((j % 12 + j / 12) as f64 / 24.) < 6f64.recip()),
        );
        assert_float_eq!(brane.grid[0].release(), 1., abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.916667, abs <= EPSILON);
        assert_float_eq!(brane.grid[72].release(), 0., abs <= EPSILON);
    }
}
