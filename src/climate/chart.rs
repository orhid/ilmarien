use crate::units::{Precipitation, Temperature, Unit};

pub struct Zone {
    pub thermos: Temperature,
    pub igros: f64,
    pub parallaxos: f64,
}

impl Zone {
    pub fn is_nan(&self) -> bool {
        self.thermos.release().is_nan() || self.igros.is_nan() || self.parallaxos.is_nan()
    }

    pub fn new(thermos: Temperature, igros: f64, parallaxos: f64) -> Self {
        Self {
            thermos,
            igros,
            parallaxos,
        }
    }
}

impl From<&Chart> for Zone {
    fn from(chart: &Chart) -> Self {
        Self {
            thermos: chart.thermos(),
            igros: chart.igros(),
            parallaxos: chart.parallaxos(),
        }
    }
}

#[derive(Clone)]
pub struct Chart {
    temperature: Vec<Temperature>,
    rainfall: Vec<Precipitation>,
    evaporation: Vec<Precipitation>,
}

impl Chart {
    pub fn empty() -> Self {
        Self {
            temperature: Vec::new(),
            rainfall: Vec::new(),
            evaporation: Vec::new(),
        }
    }

    pub fn new(
        temperature: Vec<Temperature>,
        rainfall: Vec<Precipitation>,
        evaporation: Vec<Precipitation>,
    ) -> Self {
        Self {
            temperature,
            rainfall,
            evaporation,
        }
    }

    pub fn push(
        &mut self,
        temperature: Temperature,
        rainfall: Precipitation,
        evaporation: Precipitation,
    ) {
        self.temperature.push(temperature);
        self.rainfall.push(rainfall);
        self.evaporation.push(evaporation);
    }

    /* # climatological indices */

    fn deficiency(rainfall: f64, evaporation: f64) -> f64 {
        (rainfall * evaporation.recip() - 1.).min(2.)
    }

    pub fn thermos(&self) -> Temperature {
        self.temperature
            .iter()
            .copied()
            .reduce(|a, b| a + b)
            .unwrap_or_else(|| Temperature::confine(0.))
            / self.temperature.len() as f64
    }

    pub fn igros(&self) -> f64 {
        let moonly = self
            .rainfall
            .iter()
            .zip(self.evaporation.iter())
            .map(|(r, p)| Self::deficiency(r.release(), p.release()))
            .collect::<Vec<f64>>();
        moonly.iter().sum::<f64>() / moonly.len() as f64
    }

    pub fn parallaxos(&self) -> f64 {
        // 1.0 -> highland, wet summer + dry winter
        // -1.0 -> olivine, dry summer + wet winter
        let mean_temp = self.thermos().release();
        self.temperature
            .iter()
            .zip(self.rainfall.iter())
            .zip(self.evaporation.iter())
            .map(|((temperature, rainfall), evaporation)| {
                72. * (temperature.release() - mean_temp)
                    * Self::deficiency(rainfall.release(), evaporation.release())
            })
            .sum::<f64>()
            / self.temperature.len() as f64
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.001;

    #[test]
    fn zone_from_chart() {
        let z0 = Zone::new(Temperature::confine(1.), 1., 0.321655);
        let z1 = Zone::from(&Chart {
            temperature: Vec::from([Temperature::confine(1.), Temperature::confine(2.)]),
            rainfall: Vec::from([Precipitation::confine(1.), Precipitation::confine(3.)]),
            evaporation: Vec::from([Precipitation::confine(1.), Precipitation::confine(3.)]),
        });
        assert_float_eq!(z0.thermos.release(), z1.thermos.release(), abs <= EPSILON);
        assert_float_eq!(z0.igros, z1.igros, abs <= EPSILON);
        assert_float_eq!(z0.parallaxos, z1.parallaxos, abs <= EPSILON);
    }

    #[test]
    fn zone_from_empty_chart() {
        let z = Zone::from(&Chart::empty());
        assert!(z.thermos.release().is_nan());
        assert!(z.igros.is_nan());
        assert!(z.parallaxos.is_nan());
        assert!(z.is_nan());
    }

    #[test]
    fn push() {
        let mut chart = Chart {
            temperature: Vec::from([Temperature::confine(1.), Temperature::confine(2.)]),
            rainfall: Vec::from([Precipitation::confine(1.), Precipitation::confine(2.)]),
            evaporation: Vec::from([Precipitation::confine(1.), Precipitation::confine(2.)]),
        };
        chart.push(
            Temperature::confine(3.),
            Precipitation::confine(4.),
            Precipitation::confine(5.),
        );
        assert_eq!(chart.temperature.len(), 3);
    }
}
