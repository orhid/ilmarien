/// this module contains the Zone and Chart Structs
/// which are used to classify points in space into climate types
use crate::{climate::vegetation::Vege, vars::TEMP_RANGE};
use ord_subset::OrdSubsetIterExt;
use std::collections::VecDeque;

// this is fine

pub struct Zone {
    aridity: f64,
    swing: f64,
    tmin: f64,
    tmax: f64,
}

impl Zone {
    fn is_nan(&self) -> bool {
        self.aridity.is_nan() && self.swing.is_nan() && self.tmin.is_nan() && self.tmax.is_nan()
    }
    pub fn new(aridity: f64, swing: f64, tmin: f64, tmax: f64) -> Self {
        Self {
            aridity,
            swing,
            tmin,
            tmax,
        }
    }

    pub fn dist(&self, other: &Zone) -> f64 {
        (self.aridity - other.aridity).abs() * 1.44
            + (self.swing - other.swing).abs() / 2.0
            + (self.tmin - other.tmin).abs() / TEMP_RANGE
            + (self.tmax - other.tmax).abs() / TEMP_RANGE
    }

    pub fn vege(&self) -> Vege {
        if self.is_nan() {
            Vege::Stone
        } else {
            Vege::array()
                .into_iter()
                .ord_subset_min_by_key(|vege| self.dist(&Zone::from(vege)))
                .unwrap()
        }
    }
}

impl From<&Chart> for Zone {
    fn from(chart: &Chart) -> Self {
        Self {
            aridity: chart.aridity(),
            swing: chart.swing(),
            tmin: chart.tmin(),
            tmax: chart.tmax(),
        }
    }
}

#[derive(Clone)]
pub struct Chart {
    heat: VecDeque<f64>,
    rain: VecDeque<f64>,
    peva: VecDeque<f64>,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            heat: VecDeque::new(),
            rain: VecDeque::new(),
            peva: VecDeque::new(),
        }
    }

    pub fn push(&mut self, heat: f64, rain: f64, peva: f64, cycle: usize) {
        self.heat.push_back(heat);
        self.rain.push_back(rain);
        self.peva.push_back(peva);

        if self.heat.len() > cycle {
            self.heat.pop_front();
        }
        if self.rain.len() > cycle {
            self.rain.pop_front();
        }
        if self.peva.len() > cycle {
            self.peva.pop_front();
        }
    }

    fn aridity(&self) -> f64 {
        self.rain.iter().sum::<f64>() / self.peva.iter().sum::<f64>()
    }

    fn swing(&self) -> f64 {
        // 1.0 -> highland
        // -1.0 -> olivine
        let mheat = self.heat.iter().sum::<f64>() / self.heat.len() as f64;
        let mut heat_filter = self.heat.iter().map(|&h| h > mheat);
        let mut hrain = self.rain.clone();
        hrain.retain(|_| heat_filter.next().unwrap());
        2.0 * (hrain.iter().sum::<f64>() / self.rain.iter().sum::<f64>()).powf(1.44) - 1.0
    }

    fn tmin(&self) -> f64 {
        *self.heat.iter().ord_subset_min().unwrap_or(&f64::NAN)
    }

    fn tmax(&self) -> f64 {
        *self.heat.iter().ord_subset_max().unwrap_or(&f64::NAN)
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    #[test]
    fn dist() {
        let z0 = Zone::new(0.0, 0.0, 0.0, 0.0);
        let z1 = Zone::new(1.0, 2.0, TEMP_RANGE, TEMP_RANGE);
        assert_float_eq!(z0.dist(&z1), 4.44, abs <= EPSILON);
    }

    #[test]
    fn zone_from_chart() {
        let z0 = Zone::new(1.0, 0.321655, 1.0, 2.0);
        let z1 = Zone::from(&Chart {
            heat: VecDeque::from([1.0, 2.0]),
            rain: VecDeque::from([1.0, 3.0]),
            peva: VecDeque::from([1.0, 3.0]),
        });
        assert_float_eq!(z0.dist(&z1), 0.0, abs <= EPSILON);
    }

    #[test]
    fn zone_from_empty_chart() {
        let z = Zone::from(&Chart::new());
        assert!(z.aridity.is_nan());
        assert!(z.swing.is_nan());
        assert!(z.tmin.is_nan());
        assert!(z.tmax.is_nan());
    }

    #[test]
    fn push() {
        let mut chart = Chart {
            heat: VecDeque::from([1.0, 2.0]),
            rain: VecDeque::from([1.0, 2.0]),
            peva: VecDeque::from([1.0, 2.0]),
        };
        chart.push(3.0, 4.0, 5.0, 3);
        assert_eq!(chart.heat.len(), 3);
        assert_float_eq!(*chart.heat.back().unwrap(), 3.0, abs <= EPSILON);
        chart.push(6.0, 7.0, 8.0, 3);
        assert_eq!(chart.heat.len(), 3);
        assert_float_eq!(*chart.heat.front().unwrap(), 2.0, abs <= EPSILON);
        assert_float_eq!(*chart.heat.back().unwrap(), 6.0, abs <= EPSILON);
    }

    #[test]
    fn aridity() {
        let chart = Chart {
            heat: VecDeque::new(),
            rain: VecDeque::from([1.0, 1.0, 1.0]),
            peva: VecDeque::from([2.0, 2.0, 2.0]),
        };
        assert_float_eq!(chart.aridity(), 0.5, abs <= EPSILON);
    }

    #[test]
    fn swing() {
        let chart = Chart {
            heat: VecDeque::from([1.0, 3.0]),
            rain: VecDeque::from([1.0, 3.0]),
            peva: VecDeque::new(),
        };
        assert_float_eq!(chart.swing(), 0.321655, abs <= EPSILON);
    }

    #[test]
    fn tminmax() {
        let chart = Chart {
            heat: VecDeque::from([1.0, 3.0]),
            rain: VecDeque::new(),
            peva: VecDeque::new(),
        };
        assert_float_eq!(chart.tmin(), 1.0, abs <= EPSILON);
        assert_float_eq!(chart.tmax(), 3.0, abs <= EPSILON);
    }

    #[test]
    fn koppen() {
        assert_eq!(
            Zone::new(f64::NAN, f64::NAN, f64::NAN, f64::NAN).vege(),
            Vege::Stone
        );
        assert_eq!(Zone::new(0.0, 0.0, -6.0, -6.0).vege(), Vege::Stone);
        assert_eq!(Zone::new(0.0, 0.0, 2.0, 10.0).vege(), Vege::Stone);
        assert_eq!(Zone::new(0.0, 0.0, 10.0, 26.0).vege(), Vege::Stone);
        assert_eq!(Zone::new(0.0, 0.0, 18.0, 32.0).vege(), Vege::Sand);
        assert_eq!(Zone::new(0.0, 0.0, 32.0, 32.0).vege(), Vege::Sand);

        assert_eq!(Zone::new(0.3, 0.0, -6.0, -6.0).vege(), Vege::Frost);
        assert_eq!(Zone::new(0.3, 0.0, -6.0, 2.0).vege(), Vege::Frost);
        assert_eq!(Zone::new(0.3, 0.0, -6.0, 10.0).vege(), Vege::Tundra);
        assert_eq!(Zone::new(0.3, 0.0, -6.0, 32.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 2.0, 2.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 2.0, 18.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 2.0, 32.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 10.0, 10.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 10.0, 26.0).vege(), Vege::Prairie);
        assert_eq!(Zone::new(0.3, 0.0, 10.0, 32.0).vege(), Vege::Savanna);
        assert_eq!(Zone::new(0.3, 0.0, 18.0, 26.0).vege(), Vege::Savanna);
        assert_eq!(Zone::new(0.3, 0.0, 26.0, 26.0).vege(), Vege::Sand);
        assert_eq!(Zone::new(0.3, 0.0, 32.0, 32.0).vege(), Vege::Sand);

        assert_eq!(Zone::new(0.6, 0.0, -6.0, -6.0).vege(), Vege::Frost);
        assert_eq!(Zone::new(0.6, 0.0, -6.0, 2.0).vege(), Vege::Frost);
        assert_eq!(Zone::new(0.6, 0.0, -6.0, 10.0).vege(), Vege::Tundra);
        assert_eq!(Zone::new(0.6, 0.0, -6.0, 32.0).vege(), Vege::Tundra);
        assert_eq!(Zone::new(0.6, 0.0, 2.0, 2.0).vege(), Vege::Tundra);
        assert_eq!(Zone::new(0.6, 0.0, 10.0, 32.0).vege(), Vege::Shrub);
        assert_eq!(Zone::new(0.6, 0.0, 18.0, 18.0).vege(), Vege::Shrub);
        assert_eq!(Zone::new(0.6, 0.0, 18.0, 32.0).vege(), Vege::Savanna);
        assert_eq!(Zone::new(0.6, 0.0, 32.0, 32.0).vege(), Vege::Savanna);

        assert_eq!(Zone::new(1.2, 0.0, -6.0, -6.0).vege(), Vege::Frost);
        assert_eq!(Zone::new(1.2, 0.0, -6.0, 2.0).vege(), Vege::Taiga);
        assert_eq!(Zone::new(1.2, 0.0, -6.0, 10.0).vege(), Vege::Taiga);
        assert_eq!(Zone::new(1.2, 0.0, -6.0, 32.0).vege(), Vege::Coniferous);
        assert_eq!(Zone::new(1.2, 0.0, 2.0, 2.0).vege(), Vege::Taiga);
        assert_eq!(Zone::new(1.2, 0.0, 2.0, 32.0).vege(), Vege::Coniferous);
        assert_eq!(Zone::new(1.2, 0.0, 10.0, 32.0).vege(), Vege::Decideous);
        assert_eq!(Zone::new(1.2, 0.0, 18.0, 26.0).vege(), Vege::Broadleaf);
        assert_eq!(Zone::new(1.2, 0.0, 32.0, 32.0).vege(), Vege::Broadleaf);
    }
}
