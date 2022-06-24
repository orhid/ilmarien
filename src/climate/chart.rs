use crate::units::{Precipitation, Temperature, Unit};

pub struct Zone {
    pub aridity: f64,
    pub swing: f64,
    pub tmin: Temperature,
    pub tmax: Temperature,
}

impl Zone {
    pub fn is_nan(&self) -> bool {
        self.aridity.is_nan()
            || self.swing.is_nan()
            || self.tmin.release().is_nan()
            || self.tmax.release().is_nan()
    }

    pub fn new(aridity: f64, swing: f64, tmin: Temperature, tmax: Temperature) -> Self {
        Self {
            aridity,
            swing,
            tmin,
            tmax,
        }
    }

    /*
    fn dist(&self, other: &Zone) -> f64 {
        (self.aridity - other.aridity).abs() * 2.16
            + (self.swing - other.swing).abs() / 2.0
            + (self.tmin - other.tmin).abs() / TMP_RANGE
            + (self.tmax - other.tmax).abs() / TMP_RANGE
    }
    */
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
    heat: Vec<Temperature>,
    rain: Vec<Precipitation>,
    pevt: Vec<Precipitation>,
}

impl Chart {
    pub fn empty() -> Self {
        Self {
            heat: Vec::new(),
            rain: Vec::new(),
            pevt: Vec::new(),
        }
    }

    pub fn new(heat: Vec<Temperature>, rain: Vec<Precipitation>, pevt: Vec<Precipitation>) -> Self {
        Self { heat, rain, pevt }
    }

    pub fn push(&mut self, heat: Temperature, rain: Precipitation, pevt: Precipitation) {
        self.heat.push(heat);
        self.rain.push(rain);
        self.pevt.push(pevt);
    }

    /// fucking backwards defined aridity index
    fn aridity_index(&self) -> f64 {
        self.rain.iter().map(|value| value.release()).sum::<f64>()
            * self
                .pevt
                .iter()
                .map(|value| value.release())
                .sum::<f64>()
                .recip()
    }

    pub fn aridity(&self) -> f64 {
        self.aridity_index()
    }

    pub fn swing(&self) -> f64 {
        // 1.0 -> highland
        // -1.0 -> olivine
        let mheat = Temperature::confine(
            self.heat.iter().map(|value| value.release()).sum::<f64>() / self.heat.len() as f64,
        );
        let mut heat_filter = self.heat.iter().map(|&h| h > mheat);
        let mut hrain = self.rain.clone();
        hrain.retain(|_| heat_filter.next().unwrap());
        2.0 * (hrain.iter().map(|value| value.release()).sum::<f64>()
            / self.rain.iter().map(|value| value.release()).sum::<f64>())
        .powf(1.44)
            - 1.0
    }

    pub fn tmin(&self) -> Temperature {
        *self
            .heat
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&Temperature::confine(f64::NAN))
    }

    pub fn tmax(&self) -> Temperature {
        *self
            .heat
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&Temperature::confine(f64::NAN))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    /*
    #[test]
    fn dist() {
        let z0 = Zone::new(0.0, 0.0, 0.0, 0.0);
        let z1 = Zone::new(1.0, 2.0, TMP_RANGE, TMP_RANGE);
        assert_float_eq!(z0.dist(&z1), 5.16, abs <= EPSILON);
    }
    */

    #[test]
    fn zone_from_chart() {
        let z0 = Zone::new(
            1.,
            0.321655,
            Temperature::confine(1.),
            Temperature::confine(2.),
        );
        let z1 = Zone::from(&Chart {
            heat: Vec::from([Temperature::confine(1.), Temperature::confine(2.)]),
            rain: Vec::from([Precipitation::confine(1.), Precipitation::confine(3.)]),
            pevt: Vec::from([Precipitation::confine(1.), Precipitation::confine(3.)]),
        });
        assert_float_eq!(z0.aridity, z1.aridity, abs <= EPSILON);
        assert_float_eq!(z0.swing, z1.swing, abs <= EPSILON);
        assert_float_eq!(z0.tmin.release(), z1.tmin.release(), abs <= EPSILON);
        assert_float_eq!(z0.tmax.release(), z1.tmax.release(), abs <= EPSILON);
    }

    #[test]
    fn zone_from_empty_chart() {
        let z = Zone::from(&Chart::empty());
        assert!(z.aridity.is_nan());
        assert!(z.swing.is_nan());
        assert!(z.tmin.release().is_nan());
        assert!(z.tmax.release().is_nan());
        assert!(z.is_nan());
    }

    #[test]
    fn push() {
        let mut chart = Chart {
            heat: Vec::from([Temperature::confine(1.), Temperature::confine(2.)]),
            rain: Vec::from([Precipitation::confine(1.), Precipitation::confine(2.)]),
            pevt: Vec::from([Precipitation::confine(1.), Precipitation::confine(2.)]),
        };
        chart.push(
            Temperature::confine(3.),
            Precipitation::confine(4.),
            Precipitation::confine(5.),
        );
        assert_eq!(chart.heat.len(), 3);
    }

    #[test]
    fn aridity() {
        let chart = Chart {
            heat: Vec::new(),
            rain: Vec::from([
                Precipitation::confine(1.),
                Precipitation::confine(1.),
                Precipitation::confine(1.),
            ]),
            pevt: Vec::from([
                Precipitation::confine(2.),
                Precipitation::confine(2.),
                Precipitation::confine(2.),
            ]),
        };
        assert_float_eq!(chart.aridity(), 0.5, abs <= EPSILON);
    }

    #[test]
    fn swing() {
        let chart = Chart {
            heat: Vec::from([Temperature::confine(1.), Temperature::confine(3.)]),
            rain: Vec::from([Precipitation::confine(1.), Precipitation::confine(3.)]),
            pevt: Vec::new(),
        };
        assert_float_eq!(chart.swing(), 0.321655, abs <= EPSILON);
    }

    #[test]
    fn tminmax() {
        let chart = Chart {
            heat: Vec::from([Temperature::confine(1.), Temperature::confine(3.)]),
            rain: Vec::new(),
            pevt: Vec::new(),
        };
        assert_float_eq!(chart.tmin().release(), 1.0, abs <= EPSILON);
        assert_float_eq!(chart.tmax().release(), 3.0, abs <= EPSILON);
    }
}
