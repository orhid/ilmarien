use crate::{
    carto::{
        brane::{Brane, Onion},
        flux::Flux,
    },
    vars::*,
};
use log::info;
use rayon::prelude::*;

/* # fabrics */

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fabric {
    Water,
    Snow,
    Ice,
    Stone,
}

impl From<Fabric> for u8 {
    fn from(surface: Fabric) -> Self {
        match surface {
            Fabric::Water => 0,
            Fabric::Snow => 1,
            Fabric::Ice => 2,
            Fabric::Stone => 3,
        }
    }
}

impl From<u8> for Fabric {
    fn from(value: u8) -> Self {
        match value {
            0 => Fabric::Water,
            1 => Fabric::Snow,
            2 => Fabric::Ice,
            3 => Fabric::Stone,
            _ => panic!(),
        }
    }
}

/* # layers */

#[derive(Clone)]
pub struct Layer {
    pub fabric: Fabric,
    pub depth: f64,
}

impl Layer {
    fn new(fabric: Fabric, depth: f64) -> Self {
        Self { fabric, depth }
    }
}

/* # cosmic onion */

type Cosmos = Onion<Layer>;

fn initialise_bedrock(bedrock: &Brane<f64>) -> Cosmos {
    info!("initialising bedrock for cosmic onion");

    let mut onion = Onion::from(
        bedrock
            .par_iter_exact()
            .map(|datum| vec![Layer::new(Fabric::Stone, bedrock.read(&datum))])
            .collect::<Vec<Vec<Layer>>>(),
    );
    onion.variable = "cosmos".to_string();
    onion
}

/// calculate the surface level model
pub fn elevation(cosmos: &Cosmos) -> Brane<f64> {
    info!("calculating elevation model");

    let mut brane = Brane::from(
        cosmos
            .par_iter_exact()
            .map(|datum| {
                cosmos
                    .iter_column(&datum)
                    .map(|layer| layer.depth)
                    .sum::<f64>()
            })
            .collect::<Vec<f64>>(),
    );
    brane.variable = "elevation".to_string();
    brane
}

/// calculate the elevation gradient
pub fn elevation_flux(elevation: &Brane<f64>) -> Flux<f64> {
    Flux::<f64>::from(elevation)
}

/// initialise the cosmic onion
pub fn initialise(bedrock: &Brane<f64>) -> Cosmos {
    info!("initialising cosmic onion");

    let mut cosmos = initialise_bedrock(bedrock);
    let elevation = elevation(&cosmos);

    for datum in cosmos.iter_exact() {
        let level = elevation.read(&datum);
        if level < INIT_OCEAN_LEVEL {
            cosmos.push(&datum, Layer::new(Fabric::Water, INIT_OCEAN_LEVEL - level));
        }
    }
    cosmos
}

/// calculate the surface type model
pub fn surface(cosmos: &Cosmos) -> Brane<Fabric> {
    info!("calculating surface model");

    let mut brane = Brane::from(
        cosmos
            .par_iter_exact()
            .map(|datum| cosmos.top(&datum).unwrap().fabric)
            .collect::<Vec<Fabric>>(),
    );
    brane.variable = "elevation".to_string();
    brane
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    #[test]
    fn initialise_bedrock_values() {
        let cosmos = initialise_bedrock(&Brane::from(vec![0.0, 0.25, 0.5, 0.75]));
        assert_eq!(cosmos.grid.len(), 4);
        let elevation = elevation(&cosmos);
        assert_float_eq!(elevation.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(elevation.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[2], 0.5, abs <= EPSILON);
        let surface = surface(&cosmos);
        assert_eq!(surface.grid[0], Fabric::Stone);
        assert_eq!(surface.grid[1], Fabric::Stone);
    }

    #[test]
    fn initialise_values() {
        let cosmos = initialise(&Brane::from(vec![0.0, 0.25, 0.5, 0.75]));
        assert_eq!(cosmos.grid[0].len(), 2);
        let elevation = elevation(&cosmos);
        assert_float_eq!(elevation.grid[0], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[2], 0.5, abs <= EPSILON);
        let surface = surface(&cosmos);
        assert_eq!(surface.grid[0], Fabric::Water);
        assert_eq!(surface.grid[1], Fabric::Stone);
    }
}
