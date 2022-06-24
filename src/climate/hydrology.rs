use crate::{
    carto::{
        brane::Brane,
        datum::DatumZa,
        //flux::Flux,
        honeycomb::HoneyCellToroidal,
    },
    units::{Elevation, Precipitation, Temperature, Unit},
};
use log::trace;
/*
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
*/
use std::collections::{HashMap, VecDeque};
//use rayon::prelude::*;

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
    elevation: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    ocean: Elevation,
) -> Brane<f64> {
    trace!("calculating continentality");
    let resolution = elevation.resolution;
    let ocean_tiles: Brane<bool> = elevation.operate_by_value_ref(|value| value < &ocean); // find ocean tiles
    let altitude =
        elevation.operate_by_value_ref(|value| if value < &ocean { ocean } else { *value });

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
                    altitude.grid[here_jndex],
                    temperature.grid[here_jndex],
                    altitude.grid[jndex],
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
    trace!("calculating potential evapotranspiration");

    temperature.operate_by_value_ref(|value| {
        Precipitation::confine(
            (value.celcius().max(0.) * Temperature::celcius_max().recip()).powi(2),
        )
    })
}

/*
/// actual amount of water released into the atmosphere
pub fn evaporation(potential: &Brane<f64>, vegetation: &Brane<Option<Vege>>) -> Brane<f64> {
    trace!("calculating evaporation model");
    Brane::from(
        (0..potential.resolution.pow(2))
            .into_par_iter()
            .map(|j| potential.grid[j].min(hydro_potential(vegetation.grid[j])))
            .collect::<Vec<f64>>(),
    )
}
*/

/* # rainfall */

pub fn rainfall(
    elevation: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    evaporation: &Brane<Precipitation>,
    ocean: Elevation,
) -> Brane<Precipitation> {
    // this took two minutes
    trace!("calculating precipitation");

    let resolution = elevation.resolution;
    let ocean_tiles: Brane<bool> = elevation.operate_by_value_ref(|value| value < &ocean); // find ocean tiles
    let altitude =
        elevation.operate_by_value_ref(|value| if value < &ocean { ocean } else { *value });

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
                        altitude.grid[neighbour_jndex],
                        temperature.grid[neighbour_jndex],
                        altitude.grid[here_jndex],
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

/*
/* ## rainfall */

const DROP_BASE: f64 = 0.04; // base amount of moisture dropped at every cell
const DROP_LEVEL_FACTOR: f64 = 0.72; // level difference multiplicative factor
const DROP_UPHILL: f64 = 0.18; // fraction of rain dropped uphill
                               //const RAIN_RADIUS_FACTOR: f64 = 54.0;
const RAIN_RADIUS_MAX: f64 = 12.0; // maximal distance from cell for rain drop
const RAIN_AMOUNT_MAX: f64 = 3.0;
const RAIN_CORRECTION: f64 = 0.42;

fn rainfall_nd(
    target_level: f64,
    node: NodeIndex,
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    gradient: &Graph<DatumZa, f64>,
    rainfall: &mut Brane<f64>,
) -> f64 {
    let datum = &gradient[node];
    let level = elevation.read(datum);
    let moisture = evaporation.read(datum)
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
    let drop: f64 =
        moisture.min(DROP_BASE + DROP_LEVEL_FACTOR * (target_level - level).max(0.0) * moisture);

    let radius: i32 = RAIN_RADIUS_MAX as i32;
    let volume = ball_cone_volume(radius) as f64;
    for nbr in datum.ball_toroidal(radius, rainfall.resolution as i32) {
        rainfall.grid[nbr.unravel(rainfall.resolution)] = (rainfall.grid
            [nbr.unravel(rainfall.resolution)]
            + drop
                * if elevation.read(&nbr) < target_level {
                    1.0
                } else {
                    DROP_UPHILL
                }
                * (radius - datum.dist_toroidal(&nbr, rainfall.resolution as i32) + 1) as f64
                * volume.recip())
        .min(RAIN_AMOUNT_MAX);
    }
    moisture - drop
}

fn rainfall_naive(
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    wind: &Flux<f64>,
) -> Brane<f64> {
    // this is slightly slower than diffusion, although it does look better
    // unfortunately this is also hugely resolution dependent
    //      higher resolutions are much much wetter
    // a solution to both of those problems would be to calculate rain at a set resolution
    //      and then use regression to interpolate to a higher resolution

    let mut rainfall = Brane::<f64>::zeros(elevation.resolution);
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
    let correction = RAIN_CORRECTION
        * evaporation.grid.iter().sum::<f64>()
        * rainfall.grid.iter().sum::<f64>().recip();

    rainfall * correction
}

const RAIN_BLOW: f64 = 2.16;

/// calculate the amount of rainfall reaching the surface
pub fn rainfall(
    elevation: &Brane<f64>,
    evaporation: &Brane<f64>,
    wind: &Flux<f64>,
    continentality: &Brane<f64>,
) -> Brane<f64> {
    trace!("calculating rainfall");
    let rainfall_naive = rainfall_naive(elevation, evaporation, wind);
    Brane::from(
        (0..rainfall_naive.resolution.pow(2))
            .into_par_iter()
            .map(|j| {
                RAIN_BLOW * rainfall_naive.grid[j] * 8f64.recip().max(1.0 - continentality.grid[j])
            })
            .collect::<Vec<f64>>(),
    )
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
    shed.insert(datum, moisture);
    moisture
}

/// calculate the amount of water flowing down to every datum
pub fn shed(elevation_flux: &Flux<f64>, rainfall: &Brane<f64>) -> Brane<f64> {
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
    use crate::carto::brane::Resolution;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0000_01;

    #[test]
    fn evapotranspiration_potential_values() {
        let brane = evapotranspiration_potential(&Brane::new(
            (0..36)
                .map(|j| Temperature::confine(j as f64 / 36.))
                .collect::<Vec<Temperature>>(),
            Resolution::confine(6),
        ));
        assert_float_eq!(brane.grid[0].release(), 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.012345, abs <= EPSILON);
        assert_float_eq!(brane.grid[32].release(), 0.067215, abs <= EPSILON);
    }

    /*
    #[test]
    fn evaporation_values() {
        let brane = evaporation(
            &Brane::from(vec![0.0, 1.0, 0.1, 0.3]),
            &Brane::from(vec![None, None, Some(Vege::Prairie), Some(Vege::Prairie)]),
        );
        assert_float_eq!(brane.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(brane.grid[1], 0.84, abs <= EPSILON);
        assert_float_eq!(brane.grid[2], 0.1, abs <= EPSILON);
        assert_float_eq!(brane.grid[3], 0.18, abs <= EPSILON);
    }

    #[test]
    fn rainfall_values() {
        let brane = rainfall_naive(
            &Brane::from(
                (0..144)
                    .map(|j| (j % 12 + j / 12) as f64 / 24.0)
                    .collect::<Vec<f64>>(),
            ),
            &Brane::from(vec![1.0; 144]),
            &Flux::<f64>::from(&Brane::from(
                (0..144)
                    .map(|j| (24 - (j % 12 + j / 12)) as f64 / 24.0)
                    .collect::<Vec<f64>>(),
            )),
        );
        assert_float_eq!(brane.grid[0], 0.531359, abs <= EPSILON);
        assert_float_eq!(brane.grid[12], 0.504252, abs <= EPSILON);
        assert_float_eq!(brane.grid[72], 0.500678, abs <= EPSILON);
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
    */
}
