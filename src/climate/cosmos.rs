use crate::{
    carto::{
        brane::{Brane, Onion},
        datum::DatumZa,
        flux::Flux,
    },
    climate::radiation::lapse,
    util::diffusion::reflow,
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

/* # layers */

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Layer {
    pub fabric: Fabric,
    pub depth: f64,
}

impl Layer {
    pub fn new(fabric: Fabric, depth: f64) -> Self {
        Self { fabric, depth }
    }
}

/* # cosmic onion */

pub type Cosmos = Onion<Layer>;

impl Cosmos {
    pub fn simplify_columns(&mut self) {
        self.sort_columns();
        for column in &mut self.grid {
            let mut j = 0;
            while j < column.len() - 1 {
                if column[j].fabric == column[j + 1].fabric {
                    column[j] = Layer::new(
                        column[j].fabric,
                        column[j].depth + column.remove(j + 1).depth,
                    );
                } else {
                    j += 1;
                }
            }
        }
    }

    pub fn solidify_snow(&mut self) {
        for column in &mut self.grid {
            if column.last().unwrap().fabric == Fabric::Snow {
                let depth = column.pop().unwrap().depth;
                column.push(Layer::new(Fabric::Ice, depth));
            }
        }
    }

    pub fn form_glaciers(&mut self, temperature: &Brane<f64>) -> Brane<f64> {
        // assumes one has just solidified all snow, so only ice will be present
        let mut icemelt = Brane::<f64>::zeros(self.resolution);
        let elev = self.elevation();
        let res = self.resolution;
        for datum in (0..res.pow(2)).map(|j| DatumZa::enravel(j, res)) {
            let tempdif =
                temperature.get(&datum.cast(self.resolution)) - lapse(elev.read(&datum)) - 273.0;
            let index = datum.unravel(self.resolution);
            let column = &mut self.grid[index];
            let mut potential = tempdif.abs().sqrt() * EVA_RATE;
            if tempdif > 0.0 {
                if column.last().unwrap().fabric == Fabric::Ice {
                    let mut ice = column.pop().unwrap();
                    if ice.depth > potential {
                        ice.depth -= potential;
                        column.push(ice.clone());
                    } else {
                        potential = ice.depth;
                    }
                    icemelt.grid[index] = potential * EVA_RATE.recip();
                }
            } else {
                let mut oldice = 0.0;
                if column.last().unwrap().fabric == Fabric::Ice {
                    oldice = column.pop().unwrap().depth;
                }
                if column.last().unwrap().fabric == Fabric::Water {
                    let mut water = column.pop().unwrap();
                    if water.depth > potential {
                        water.depth -= potential;
                        column.push(water.clone());
                    } else {
                        potential = water.depth;
                    }
                    column.push(Layer::new(Fabric::Ice, potential + oldice));
                }
            }
        }
        icemelt
    }

    /*
    pub fn snowfall(&mut self, rainfall: &mut Brane<f64>, temperature: &Brane<f64>) {
        let elev = self.elevation();
        for datum in (0..self.resolution.pow(2)).map(|j| DatumZa::enravel(j, self.resolution)) {
            let tempdif =
                temperature.get(&datum.cast(self.resolution)) - lapse(elev.read(&datum)) - 273.0;
            if tempdif < 0.0 {
                let index = datum.unravel(self.resolution);
            }
        }
    }
    */

    fn lift_glaciers(&mut self) -> Self {
        // will assume that colums are already simplified
        let mut glaciers = Cosmos::from(vec![Vec::<Layer>::new(); self.resolution.pow(2)]);
        let res = self.resolution;
        for datum in (0..res.pow(2)).map(|j| DatumZa::enravel(j, res)) {
            let mut j = 0;
            let index = datum.unravel(self.resolution);
            let column = &mut self.grid[index];
            while j < column.len() - 1 {
                if column[j].fabric == Fabric::Ice || column[j].fabric == Fabric::Snow {
                    glaciers.grid[index] = column.split_off(j);
                    break;
                }
                j += 1;
            }
        }
        glaciers
    }

    fn drop_glaciers(&mut self, glaciers: &mut Self) {
        let res = self.resolution;
        for datum in (0..res.pow(2)).map(|j| DatumZa::enravel(j, res)) {
            let index = datum.unravel(self.resolution);
            self.grid[index].append(&mut glaciers.grid[index]);
        }
    }

    fn discard_oceans(&mut self) {
        for column in &mut self.grid {
            if column.last().unwrap().fabric == Fabric::Water {
                column.pop();
            }
        }
    }

    fn place_oceans(&mut self, ocean_elevation: &Brane<f64>) {
        let rock_elevation = self.elevation();

        let res = self.resolution;
        for datum in (0..res.pow(2)).map(|j| DatumZa::enravel(j, res)) {
            let rock_level = rock_elevation.read(&datum);
            let ocean_level = ocean_elevation.read(&datum);
            if rock_level < ocean_level {
                self.push(&datum, Layer::new(Fabric::Water, ocean_level - rock_level));
            }
        }
    }

    pub fn reflow_oceans(&mut self) {
        trace!("reflowing oceans");
        let mut glaciers = self.lift_glaciers();
        for _ in 0..self.resolution {
            let mut elevation_map = self.elevation();
            let surface_map = self.surface();
            self.discard_oceans();
            elevation_map.grid = (0..elevation_map.resolution.pow(2))
                .into_par_iter()
                .map(|j| {
                    reflow(
                        &DatumZa::enravel(j, elevation_map.resolution),
                        &elevation_map,
                        &surface_map,
                    )
                })
                .collect::<Vec<f64>>();
            self.place_oceans(&elevation_map);
        }
        self.drop_glaciers(&mut glaciers);
    }

    fn initialise_bedrock(bedrock: &Brane<f64>) -> Self {
        trace!("initialising bedrock for cosmic onion");

        let mut onion = Self::from(
            (0..bedrock.resolution.pow(2))
                .into_par_iter()
                .map(|j| {
                    vec![Layer::new(
                        Fabric::Stone,
                        bedrock.read(&DatumZa::enravel(j, bedrock.resolution)),
                    )]
                })
                .collect::<Vec<Vec<Layer>>>(),
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
                .map(|j| {
                    self.iter_column(&DatumZa::enravel(j, self.resolution))
                        .map(|layer| layer.depth)
                        .sum::<f64>()
                })
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
                .map(|j| {
                    self.iter_column(&DatumZa::enravel(j, self.resolution))
                        .filter(|layer| layer.fabric == Fabric::Stone)
                        .map(|layer| layer.depth)
                        .sum::<f64>()
                })
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
                .map(|j| {
                    self.top(&DatumZa::enravel(j, self.resolution))
                        .unwrap()
                        .fabric
                })
                .collect::<Vec<Fabric>>(),
        );
        brane.variable = "elevation".to_string();
        brane
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

        assert!(Layer::new(Fabric::Stone, 1.0) < Layer::new(Fabric::Water, 0.5));
        assert!(Layer::new(Fabric::Stone, 0.5) < Layer::new(Fabric::Stone, 1.0));
    }

    #[test]
    fn cosmos_simplify_columns() {
        let mut cosmos = Brane::from(vec![vec![
            Layer::new(Fabric::Stone, 1.0),
            Layer::new(Fabric::Stone, 1.0),
            Layer::new(Fabric::Ice, 1.0),
        ]]);
        cosmos.simplify_columns();
        assert_eq!(cosmos.grid[0].len(), 2);
        assert_eq!(cosmos.grid[0][0].fabric, Fabric::Stone);
        assert_eq!(cosmos.grid[0][1].fabric, Fabric::Ice);
        assert_float_eq!(cosmos.grid[0][0].depth, 2.0, abs <= EPSILON);
    }

    #[test]
    fn cosmos_solidify_snow() {
        let mut cosmos = Brane::from(vec![vec![
            Layer::new(Fabric::Stone, 1.0),
            Layer::new(Fabric::Snow, 1.0),
        ]]);
        cosmos.solidify_snow();
        assert_eq!(cosmos.grid[0].len(), 2);
        assert_eq!(cosmos.grid[0][0].fabric, Fabric::Stone);
        assert_eq!(cosmos.grid[0][1].fabric, Fabric::Ice);
        assert_float_eq!(cosmos.grid[0][1].depth, 1.0, abs <= EPSILON);
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

    #[test]
    fn cosmos_lift_drop_glaciers() {
        let mut cosmos = Brane::from(vec![vec![
            Layer::new(Fabric::Stone, 1.0),
            Layer::new(Fabric::Ice, 1.0),
            Layer::new(Fabric::Snow, 1.0),
        ]]);
        let mut glaciers = cosmos.lift_glaciers();
        assert_eq!(cosmos.grid[0].len(), 1);
        assert_eq!(cosmos.grid[0][0].fabric, Fabric::Stone);
        cosmos.drop_glaciers(&mut glaciers);
        assert_eq!(cosmos.grid[0].len(), 3);
        assert_eq!(cosmos.grid[0][0].fabric, Fabric::Stone);
        assert_eq!(cosmos.grid[0][1].fabric, Fabric::Ice);
        assert_eq!(cosmos.grid[0][2].fabric, Fabric::Snow);
    }

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
        assert_eq!(cosmos.grid[0].len(), 2);
        let elevation = cosmos.elevation();
        assert_float_eq!(elevation.grid[0], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[1], 0.25, abs <= EPSILON);
        assert_float_eq!(elevation.grid[2], 0.5, abs <= EPSILON);
        let surface = cosmos.surface();
        assert_eq!(surface.grid[0], Fabric::Water);
        assert_eq!(surface.grid[1], Fabric::Stone);
    }
}
