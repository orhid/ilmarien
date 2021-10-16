use crate::carto::{brane::Brane, datum::DatumZa};
use log::trace;
use ord_subset::OrdSubsetIterExt;
use rayon::prelude::*;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone)]
enum Type {
    Continental,
    Coastal,
}

enum Heat {
    Polar,
    Tundra,
    SubPolar,
    Temperate(Type),
    SubTropical(Type),
    Tropical,
}

enum Rain {
    Arid,
    SubArid,
    Highland,
    Olivine,
    Maritime,
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
            tmax: f64::NAN,
            tmed: f64::NAN,
            tmin: f64::NAN,
            rmed: f64::NAN,
            rmin: f64::NAN,
            rhot: f64::NAN,
            rcol: f64::NAN,
        }
    }

    pub fn update(&mut self, temp: f64, rain: f64) {
        if temp > self.tmax || self.tmax.is_nan() {
            self.tmax = temp;
            self.rhot = rain;
        }
        if temp < self.tmin || self.tmin.is_nan() {
            self.tmin = temp;
            self.rcol = rain;
        }
        if self.tmed.is_nan() {
            self.tmed = temp;
        } else {
            self.tmed = (self.tmax + self.tmin + 3.0 * self.tmed + temp) * 6.0f64.recip();
        }
        if rain < self.rmin || self.rmin.is_nan() {
            self.rmin = rain;
        }
        if self.rmed.is_nan() {
            self.rmed = rain;
        } else {
            self.rmed = (self.rhot + self.rcol + 3.0 * self.rmed + rain) * 6.0f64.recip();
        }
    }

    fn find_type(&self) -> Type {
        if self.tmin < 8.0 {
            Type::Continental
        } else {
            Type::Coastal
        }
    }

    fn find_rain(&self) -> Rain {
        let mut threshold = self.tmed;
        let step = 9.6;
        if self.rhot > 1.44 * self.rmed {
            threshold += 2.0 * step;
        } else if self.rhot > 1.08 * self.rmed {
            threshold += 1.0 * step;
        }

        if self.rmed < 0.5 * threshold {
            Rain::Arid
        } else if self.rmed < threshold {
            Rain::SubArid
        } else {
            if self.rcol > self.rhot * 1.08 {
                Rain::Highland
            } else if self.rcol < self.rhot * 0.64 {
                Rain::Olivine
            } else {
                Rain::Maritime
            }
        }
    }

    fn find_heat(&self, t: Type) -> Heat {
        if self.tmax < 0.0 {
            Heat::Polar
        } else if self.tmax < 7.0 {
            Heat::Tundra
        } else if self.tmin > 18.0 {
            Heat::Tropical
        } else if self.tmax > 21.0 {
            Heat::SubTropical(t)
        } else if self.tmin < 0.0 {
            Heat::SubPolar
        } else {
            Heat::Temperate(t)
        }
    }

    pub fn classify(&self) -> Koppen {
        let t = self.find_type();
        let rain = self.find_rain();
        let heat = self.find_heat(t.clone());
        match rain {
            Rain::Arid => match t {
                Type::Continental => Koppen::BWc,
                Type::Coastal => Koppen::BWh,
            },
            Rain::SubArid => match t {
                Type::Continental => Koppen::BSc,
                Type::Coastal => Koppen::BSh,
            },
            _ => match heat {
                Heat::Polar => Koppen::EF,
                Heat::Tundra => Koppen::ET,
                Heat::Tropical => {
                    if self.rmin > 54.0 {
                        Koppen::Af
                    } else if self.rmin > 96.0 - self.rmed {
                        Koppen::Am
                    } else {
                        Koppen::As
                    }
                }
                Heat::SubPolar => match rain {
                    Rain::Highland => Koppen::Dwd,
                    Rain::Olivine => Koppen::Dsd,
                    _ => Koppen::Dfd,
                },
                Heat::Temperate(t) => match t {
                    Type::Continental => match rain {
                        Rain::Highland => Koppen::Dwc,
                        Rain::Olivine => Koppen::Dsc,
                        _ => Koppen::Dfc,
                    },
                    Type::Coastal => match rain {
                        Rain::Highland => Koppen::Cwc,
                        Rain::Olivine => Koppen::Csc,
                        _ => Koppen::Cfc,
                    },
                },
                Heat::SubTropical(t) => match t {
                    Type::Continental => match rain {
                        Rain::Highland => Koppen::Dwa,
                        Rain::Olivine => Koppen::Dsa,
                        _ => Koppen::Dfa,
                    },
                    Type::Coastal => match rain {
                        Rain::Highland => Koppen::Cwa,
                        Rain::Olivine => Koppen::Csa,
                        _ => Koppen::Cfa,
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

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0001;

    #[test]
    fn param_update() {
        let mut kp = KopParam::zero();
        kp.update(12.0, 36.0);
        assert_float_eq!(kp.tmax, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.rhot, 36.0, abs <= EPSILON);
        assert_float_eq!(kp.tmin, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.rcol, 36.0, abs <= EPSILON);
        assert_float_eq!(kp.tmed, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.rmin, 36.0, abs <= EPSILON);
        assert_float_eq!(kp.rmed, 36.0, abs <= EPSILON);
        kp.update(18.0, 12.0);
        assert_float_eq!(kp.tmax, 18.0, abs <= EPSILON);
        assert_float_eq!(kp.rhot, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.tmin, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.rcol, 36.0, abs <= EPSILON);
        assert_float_eq!(kp.tmed, 14.0, abs <= EPSILON);
        assert_float_eq!(kp.rmin, 12.0, abs <= EPSILON);
        assert_float_eq!(kp.rmed, 28.0, abs <= EPSILON);
    }

    // should test classification
}
