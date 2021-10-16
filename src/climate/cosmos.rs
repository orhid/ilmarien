use crate::{
    carto::{brane::Brane, flux::Flux},
    climate::{koppen::KopParam, radiation::lapse},
    vars::*,
};
use log::trace;
use rayon::prelude::*;

/* # fabrics */

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fabric {
    Stone,
    Water,
    Ice,
    Snow,
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

/* # pillars */

#[derive(Clone, PartialEq, Debug)]
pub struct Pillar {
    pub bedrock: f64,
    pub ocean: f64,
    pub ice: f64,
    pub snow: f64,
    pub vege: u8, // this will be an enum
    pub kp: KopParam,
}

impl Pillar {
    pub fn zero() -> Self {
        Self {
            bedrock: 0.0,
            ocean: 0.0,
            ice: 0.0,
            snow: 0.0,
            vege: 0,
            kp: KopParam::zero(),
        }
    }

    fn bedrock(bedrock: f64) -> Self {
        Self {
            bedrock,
            ocean: 0.0,
            ice: 0.0,
            snow: 0.0,
            vege: 0,
            kp: KopParam::zero(),
        }
    }

    fn elevation(&self) -> f64 {
        self.bedrock + self.ocean + self.ice + self.snow
    }

    fn land_elevation(&self) -> f64 {
        self.bedrock + self.ice + self.snow
    }
}

impl From<&Pillar> for Fabric {
    fn from(pillar: &Pillar) -> Self {
        if pillar.snow > 0.0 {
            Fabric::Snow
        } else if pillar.ice > 0.0 {
            Fabric::Ice
        } else if pillar.ocean > 0.0 {
            Fabric::Water
        } else {
            Fabric::Stone
        }
    }
}

/* # cosmos */

pub type Cosmos = Brane<Pillar>;

impl Cosmos {
    pub fn solidify_snow(&mut self) {
        for pillar in &mut self.grid {
            if pillar.snow > 0.0 {
                pillar.ice += pillar.snow * ICE_COMP.recip();
                pillar.snow = 0.0;
            }
        }
    }

    pub fn form_glaciers(&mut self, temperature: &Brane<f64>) -> Brane<f64> {
        // assumes one has just solidified all snow, so only ice will be present
        let mut icemelt = Brane::<f64>::zeros(self.resolution);
        let elev = self.elevation();
        for index in 0..self.resolution.pow(2) {
            let tempdif = temperature.grid[index] - lapse(elev.grid[index]) - 273.0;
            let pillar = &mut self.grid[index];
            let mut potential = tempdif.abs().sqrt() * EVA_RATE;
            if tempdif > 0.0 {
                if pillar.ice > potential {
                    pillar.ice -= potential;
                } else {
                    potential = pillar.ice;
                }
                icemelt.grid[index] = potential * EVA_RATE.recip();
            } else {
                if pillar.ocean > potential {
                    pillar.ocean -= potential;
                } else {
                    potential = pillar.ocean;
                }
                pillar.ice += potential;
            }
        }
        icemelt
    }

    pub fn snowfall(&mut self, rainfall: &mut Brane<f64>, temperature: &Brane<f64>) {
        trace!("calculating snowfall");
        let elev = self.elevation();
        for index in 0..self.resolution.pow(2) {
            if temperature.grid[index] - lapse(elev.grid[index]) < 273.0 {
                self.grid[index].snow += rainfall.grid[index] * EVA_RATE * ICE_COMP;
                rainfall.grid[index] = 0.0;
            }
        }
    }

    fn place_oceans(&mut self, ocean_elevation: &Brane<f64>) {
        let rock_elevation = self.elevation();

        for index in 0..self.resolution.pow(2) {
            let rock_level = rock_elevation.grid[index];
            let ocean_level = ocean_elevation.grid[index];
            if rock_level < ocean_level {
                self.grid[index].ocean = ocean_level - rock_level;
            }
        }
    }

    fn initialise_bedrock(bedrock: &Brane<f64>) -> Self {
        trace!("initialising bedrock for cosmic onion");

        let mut onion = Self::from(
            (0..bedrock.resolution.pow(2))
                .into_par_iter()
                .map(|j| Pillar::bedrock(bedrock.grid[j]))
                .collect::<Vec<Pillar>>(),
        );
        onion.variable = "cosmos".to_string();
        onion
    }

    /// initialise the cosmic onion
    pub fn initialise(bedrock: &Brane<f64>) -> Self {
        trace!("initialising cosmic onion");

        let mut cosmos = Self::initialise_bedrock(bedrock);
        let ocean_elevation = Brane::from(vec![INIT_OCEAN_LEVEL; cosmos.resolution.pow(2)]);
        cosmos.place_oceans(&ocean_elevation);
        cosmos
    }

    /// calculate the surface level model
    pub fn elevation(&self) -> Brane<f64> {
        let mut brane = Brane::from(
            (0..self.resolution.pow(2))
                .into_par_iter()
                .map(|j| self.grid[j].elevation())
                .collect::<Vec<f64>>(),
        );
        brane.variable = "elevation".to_string();
        brane
    }

    /// calculate the elevation gradient, only using solid layers
    pub fn landflow(&self) -> Flux<f64> {
        trace!("calculating elevation gradient model");

        let mut flux = Flux::<f64>::from(&Brane::from(
            (0..self.resolution.pow(2))
                .into_par_iter()
                .map(|j| self.grid[j].land_elevation())
                .collect::<Vec<f64>>(),
        ));
        flux.variable = "elevation".to_string();
        flux
    }

    /// calculate the surface type model
    pub fn surface(&self) -> Brane<Fabric> {
        let mut brane = Brane::from(
            (0..self.resolution.pow(2))
                .into_par_iter()
                .map(|j| Fabric::from(&self.grid[j]))
                .collect::<Vec<Fabric>>(),
        );
        brane.variable = "elevation".to_string();
        brane
    }

    pub fn update_kp(
        &mut self,
        elevation: &Brane<f64>,
        temperature: &Brane<f64>,
        rainfall: &Brane<f64>,
    ) {
        for index in 0..self.resolution.pow(2) {
            self.grid[index].kp.update(
                temperature.grid[index] - lapse(elevation.grid[index]),
                rainfall.grid[index],
            );
        }
    }
}

/// calculate the elevation gradient
pub fn elevation_flux(elevation: &Brane<f64>) -> Flux<f64> {
    Flux::<f64>::from(elevation)
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    #[test]
    fn layer_order() {
        assert!(Fabric::Stone < Fabric::Water);
        assert!(Fabric::Water < Fabric::Ice);
        assert!(Fabric::Ice < Fabric::Snow);
    }

    #[test]
    fn cosmos_solidify_snow() {
        let mut cosmos = Brane::from(vec![Pillar::zero()]);
        cosmos.grid[0].snow = 1.0;
        cosmos.solidify_snow();
        assert_float_eq!(cosmos.grid[0].snow, 0.0, abs <= EPSILON);
        assert_float_eq!(cosmos.grid[0].ice, ICE_COMP.recip(), abs <= EPSILON);
    }

    /*
    #[test]
    fn cosmos_snowfall() {
        let mut cosmos = Brane::from(vec![
            vec![Layer::new(Fabric::Stone, 0.24)],
            vec![Layer::new(Fabric::Stone, 0.24)],
            vec![Layer::new(Fabric::Stone, 0.24)],
            vec![Layer::new(Fabric::Stone, 0.24)],
        ]);
        let mut rainfall = Brane::from(vec![1.0, 0.0, 1.0, 0.0]);
        cosmos.snowfall(&mut rainfall, &Brane::from(vec![432.0, 306.0, 0.0, 256.0]));
        assert_eq!(cosmos.grid[0].len(), 1);
        assert_eq!(cosmos.grid[2].len(), 2);
        assert_float_eq!(rainfall.grid[2], 0.0, abs <= EPSILON);
    }

    #[test]
    fn cosmos_form_glaciers() {
        let mut cosmos = Brane::from(vec![
            vec![
                Layer::new(Fabric::Stone, 0.24),
                Layer::new(Fabric::Ice, 0.00006),
            ],
            vec![
                Layer::new(Fabric::Stone, 0.24),
                Layer::new(Fabric::Ice, 0.06),
            ],
            vec![
                Layer::new(Fabric::Stone, 0.24),
                Layer::new(Fabric::Water, 0.00006),
            ],
            vec![
                Layer::new(Fabric::Stone, 0.24),
                Layer::new(Fabric::Water, 0.06),
            ],
        ]);
        let icemelt = cosmos.form_glaciers(&Brane::from(vec![432.0, 306.0, 0.0, 256.0]));
        assert_eq!(cosmos.grid[0].len(), 1);
        assert_eq!(cosmos.grid[1].len(), 2);
        assert_eq!(cosmos.grid[2].len(), 2);
        assert_eq!(cosmos.grid[3].len(), 3);
        assert_float_eq!(icemelt.grid[0], 0.00006 * EVA_RATE.recip(), abs <= EPSILON);
        assert!(icemelt.grid[1] > 0.0 && icemelt.grid[1] < 0.06 * EVA_RATE.recip());
    }
    */

    #[test]
    fn initialise_bedrock_values() {
        let cosmos = Cosmos::initialise_bedrock(&Brane::from(vec![0.0, 0.25, 0.5, 0.75]));
        assert_eq!(cosmos.grid.len(), 4);
        let elevation = cosmos.elevation();
        assert_float_eq!(elevation.grid[0], 0.0, abs <= EPSILON);
        assert_float_eq!(elevation.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[2], 0.5, abs <= EPSILON);
        let surface = cosmos.surface();
        assert_eq!(surface.grid[0], Fabric::Stone);
        assert_eq!(surface.grid[1], Fabric::Stone);
    }

    #[test]
    fn initialise_values() {
        let cosmos = Cosmos::initialise(&Brane::from(vec![0.0, 0.25, 0.5, 0.75]));
        let elevation = cosmos.elevation();
        assert_float_eq!(elevation.grid[0], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[2], 0.5, abs <= EPSILON);
        let surface = cosmos.surface();
        assert_eq!(surface.grid[0], Fabric::Water);
        assert_eq!(surface.grid[1], Fabric::Stone);
    }
}
