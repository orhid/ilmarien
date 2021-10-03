use crate::carto::{brane::Brane, datum::DatumZa};
use log::trace;
use ord_subset::OrdSubsetIterExt;
use rayon::prelude::*;

#[derive(Clone)]
pub enum Koppen {
    Af,
    Am,
    As,
    BSh,
    BSc,
    BWh,
    BWc,
    Cfa,
    Cfc,
    Csa,
    Csc,
    Cwa,
    Cwc,
    Dfa,
    Dfc,
    Dfd,
    Dsa,
    Dsc,
    Dsd,
    Dwa,
    Dwc,
    Dwd,
    EF,
    ET,
}

/* ## classificating functions */

enum Arid {
    Desert,
    Steppe,
    False,
}

enum Heat {
    Tropical,
    Maritime,
    Continental,
}

enum Polar {
    Glacier,
    Tundra,
    False,
}

enum Temp {
    SubTropical,
    Temperate,
    SubPolar,
}

enum Rain {
    Oceanic,
    Highland,
    Mediterranean,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KopParam {
    pub tmax: f64,
    pub tmed: f64,
    pub tmin: f64,
    pub rmed: f64,
    pub rmin: f64,
    pub rhot: f64,
    pub rcol: f64,
}

impl KopParam {
    pub fn zero() -> Self {
        Self {
            tmax: 0.0,
            tmed: 0.0,
            tmin: 0.0,
            rmed: 0.0,
            rmin: 0.0,
            rhot: 0.0,
            rcol: 0.0,
        }
    }

    fn find_arid(&self) -> Arid {
        //let mut threshold = self.tmed * 0.84;
        let mut threshold = self.tmed;
        let step = 9.6;
        if self.rhot > 1.44 * self.rmed {
            threshold += 2.0 * step;
        } else if self.rhot > 1.08 * self.rmed {
            threshold += 1.0 * step;
        }

        if self.rmed < 0.5 * threshold {
            Arid::Desert
        } else if self.rmed < threshold {
            Arid::Steppe
        } else {
            Arid::False
        }
    }

    fn find_heat(&self) -> Heat {
        if self.tmin > 18.0 {
            Heat::Tropical
        } else if self.tmin < 8.0 {
            Heat::Continental
        } else {
            Heat::Maritime
        }
    }

    fn find_polar(&self) -> Polar {
        if self.tmax < 0.0 {
            Polar::Glacier
        } else if self.tmax < 7.0 {
            Polar::Tundra
        } else {
            Polar::False
        }
    }

    fn class_temp(&self) -> Temp {
        if self.tmax > 21.0 {
            Temp::SubTropical
        } else if self.tmin > 0.0 {
            Temp::Temperate
        } else {
            Temp::SubPolar
        }
    }

    fn class_rain(&self) -> Rain {
        if self.rcol > self.rhot * 1.08 {
            Rain::Highland
        } else if self.rcol < self.rhot * 0.64 {
            Rain::Mediterranean
        } else {
            Rain::Oceanic
        }
    }

    fn classify(&self) -> Koppen {
        match self.find_arid() {
            Arid::Desert => match self.find_heat() {
                Heat::Continental => Koppen::BWc,
                _ => Koppen::BWh,
            },
            Arid::Steppe => match self.find_heat() {
                Heat::Continental => Koppen::BSc,
                _ => Koppen::BSh,
            },
            Arid::False => match self.find_polar() {
                Polar::Glacier => Koppen::EF,
                Polar::Tundra => Koppen::ET,
                Polar::False => match self.find_heat() {
                    Heat::Tropical => {
                        if self.rmin > 54.0 {
                            Koppen::Af
                        } else if self.rmin > 96.0 - self.rmed {
                            Koppen::Am
                        } else {
                            Koppen::As
                        }
                    }
                    Heat::Maritime => match (self.class_temp(), self.class_rain()) {
                        (Temp::SubTropical, Rain::Oceanic) => Koppen::Cfa,
                        (Temp::SubTropical, Rain::Highland) => Koppen::Cwa,
                        (Temp::SubTropical, Rain::Mediterranean) => Koppen::Csa,
                        (Temp::Temperate, Rain::Oceanic) => Koppen::Cfc,
                        (Temp::Temperate, Rain::Highland) => Koppen::Cwc,
                        (Temp::Temperate, Rain::Mediterranean) => Koppen::Csc,
                        _ => panic!("Temp::SubPolar cannot appear in Heat::Maritime"),
                    },
                    Heat::Continental => match (self.class_temp(), self.class_rain()) {
                        (Temp::SubTropical, Rain::Oceanic) => Koppen::Dfa,
                        (Temp::SubTropical, Rain::Highland) => Koppen::Dwa,
                        (Temp::SubTropical, Rain::Mediterranean) => Koppen::Dsa,
                        (Temp::Temperate, Rain::Oceanic) => Koppen::Dfc,
                        (Temp::Temperate, Rain::Highland) => Koppen::Dwc,
                        (Temp::Temperate, Rain::Mediterranean) => Koppen::Dsc,
                        (Temp::SubPolar, Rain::Oceanic) => Koppen::Dfd,
                        (Temp::SubPolar, Rain::Highland) => Koppen::Dwd,
                        (Temp::SubPolar, Rain::Mediterranean) => Koppen::Dsd,
                    },
                },
            },
        }
    }
}

/* ## at datum calculation */

fn zone_dt(datum: &DatumZa, temps: &Vec<Brane<f64>>, rains: &Vec<Brane<f64>>) -> Koppen {
    let loc_temps = temps
        .iter()
        .map(|b| b.read(datum) - 273.0)
        .collect::<Vec<f64>>();
    let loc_rains = rains
        .iter()
        .map(|b| b.read(datum) * 162.0)
        .collect::<Vec<f64>>();
    KopParam {
        tmax: *loc_temps.iter().ord_subset_max().unwrap(),
        tmed: loc_temps.iter().sum::<f64>() / loc_temps.len() as f64,
        tmin: *loc_temps.iter().ord_subset_min().unwrap(),
        rmed: loc_rains.iter().sum::<f64>() / loc_rains.len() as f64,
        rmin: *loc_rains.iter().ord_subset_min().unwrap(),
        rhot: loc_rains[loc_temps
            .iter()
            .enumerate()
            .ord_subset_max_by_key(|(_, &value)| value)
            .map(|(idx, _)| idx)
            .unwrap()],
        rcol: loc_rains[loc_temps
            .iter()
            .enumerate()
            .ord_subset_min_by_key(|(_, &value)| value)
            .map(|(idx, _)| idx)
            .unwrap()],
    }
    .classify()
}

/// calculate climate type
pub fn zone(temps: &Vec<Brane<f64>>, rains: &Vec<Brane<f64>>) -> Brane<Koppen> {
    trace!("calculating koppen zones");

    let resolution = temps[0].resolution;
    let mut brane = Brane::from(
        (0..resolution.pow(2))
            .into_par_iter()
            .map(|j| zone_dt(&DatumZa::enravel(j, resolution), temps, rains))
            .collect::<Vec<Koppen>>(),
    );
    brane.variable = "koppen".to_string();
    brane
}
