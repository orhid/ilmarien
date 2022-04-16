/// this module contains the Zone and Chart Structs
/// which are used to classify points in space into climate types
use ord_subset::OrdSubsetIterExt;

const TMP_RANGE: f64 = 72.0;

pub struct Zone {
    pub aridity: f64,
    pub swing: f64,
    pub tmin: f64,
    pub tmax: f64,
}

impl Zone {
    pub fn is_nan(&self) -> bool {
        self.aridity.is_nan() || self.swing.is_nan() || self.tmin.is_nan() || self.tmax.is_nan()
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
        (self.aridity - other.aridity).abs() * 2.16
            + (self.swing - other.swing).abs() / 2.0
            + (self.tmin - other.tmin).abs() / TMP_RANGE
            + (self.tmax - other.tmax).abs() / TMP_RANGE
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
    heat: Vec<f64>,
    rain: Vec<f64>,
    peva: Vec<f64>,
}

/// exchanges heat from [0,1] range into Celcius
fn celcius(heat: f64) -> f64 {
    heat.mul_add(TMP_RANGE, -15.0)
}

impl Chart {
    pub fn new() -> Self {
        Self {
            heat: Vec::new(),
            rain: Vec::new(),
            peva: Vec::new(),
        }
    }

    pub fn push(&mut self, heat: f64, rain: f64, peva: f64) {
        self.heat.push(celcius(heat));
        self.rain.push(rain);
        self.peva.push(peva);
    }

    pub fn aridity(&self) -> f64 {
        self.rain.iter().sum::<f64>() * self.peva.iter().sum::<f64>().recip()
    }

    pub fn swing(&self) -> f64 {
        // 1.0 -> highland
        // -1.0 -> olivine
        let mheat = self.heat.iter().sum::<f64>() / self.heat.len() as f64;
        let mut heat_filter = self.heat.iter().map(|&h| h > mheat);
        let mut hrain = self.rain.clone();
        hrain.retain(|_| heat_filter.next().unwrap());
        2.0 * (hrain.iter().sum::<f64>() / self.rain.iter().sum::<f64>()).powf(1.44) - 1.0
    }

    pub fn tmin(&self) -> f64 {
        *self.heat.iter().ord_subset_min().unwrap_or(&f64::NAN)
    }

    pub fn tmax(&self) -> f64 {
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
        let z1 = Zone::new(1.0, 2.0, TMP_RANGE, TMP_RANGE);
        assert_float_eq!(z0.dist(&z1), 5.16, abs <= EPSILON);
    }

    #[test]
    fn zone_from_chart() {
        let z0 = Zone::new(1.0, 0.321655, 1.0, 2.0);
        let z1 = Zone::from(&Chart {
            heat: Vec::from([1.0, 2.0]),
            rain: Vec::from([1.0, 3.0]),
            peva: Vec::from([1.0, 3.0]),
        });
        assert_float_eq!(z0.dist(&z1), 0.0, abs <= EPSILON);
    }

    #[test]
    fn zone_from_empty_chart() {
        let z = Zone::from(&Chart::new());
        //assert!(z.aridity.is_nan());
        assert!(z.swing.is_nan());
        assert!(z.tmin.is_nan());
        assert!(z.tmax.is_nan());
        assert!(z.is_nan());
    }

    #[test]
    fn push() {
        let mut chart = Chart {
            heat: Vec::from([1.0, 2.0]),
            rain: Vec::from([1.0, 2.0]),
            peva: Vec::from([1.0, 2.0]),
        };
        chart.push(3.0, 4.0, 5.0);
        assert_eq!(chart.heat.len(), 3);
    }

    #[test]
    fn aridity() {
        let chart = Chart {
            heat: Vec::new(),
            rain: Vec::from([1.0, 1.0, 1.0]),
            peva: Vec::from([2.0, 2.0, 2.0]),
        };
        assert_float_eq!(chart.aridity(), 0.5, abs <= EPSILON);
    }

    #[test]
    fn swing() {
        let chart = Chart {
            heat: Vec::from([1.0, 3.0]),
            rain: Vec::from([1.0, 3.0]),
            peva: Vec::new(),
        };
        assert_float_eq!(chart.swing(), 0.321655, abs <= EPSILON);
    }

    #[test]
    fn tminmax() {
        let chart = Chart {
            heat: Vec::from([1.0, 3.0]),
            rain: Vec::new(),
            peva: Vec::new(),
        };
        assert_float_eq!(chart.tmin(), 1.0, abs <= EPSILON);
        assert_float_eq!(chart.tmax(), 3.0, abs <= EPSILON);
    }
}
