use crate::{
    carto::{
        brane::{Brane, Resolution},
        datum::{DatumRe, DatumZa},
        flux::Flux,
        honeycomb::{Hexagon, HoneyCellToroidal},
    },
    units::{Elevation, Precipitation, Temperature, Unit},
};
use log::trace;
use noise::{NoiseFn, OpenSimplex, Seedable};
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};
use splines::{Interpolation, Key, Spline};
use std::collections::VecDeque;

use std::f64::consts::TAU;
const SQRT3B2: f64 = 0.8660254;
pub const OCNLV: f64 = 0.333333;

/* # bedrock generation */

pub fn bedrock(seed: u32) -> Brane<Elevation> {
    // load base from file
    let elevation_base = Brane::<Elevation>::load("elevation-base".to_string()).release();

    // prepeare noise
    let noise = OpenSimplex::new().set_seed(seed);
    let amplicies = [
        2f64.recip(),
        4f64.recip(),
        8f64.recip(),
        12f64.recip(),
        24f64.recip(),
        48f64.recip(),
        96f64.recip(),
        192f64.recip(),
    ]
    .zip([3., 6., 12., 24., 48., 96., 192., 384.]);
    let toroidal_sample = |datum: &DatumRe| -> f64 {
        let fractional_brownian_motion = |x: f64, y: f64| {
            amplicies
                .map(|(amplitude, frequency)| {
                    amplitude
                        * noise.get([
                            // toroidal wrapping
                            frequency * x.cos(),
                            frequency * x.sin(),
                            frequency * SQRT3B2 * y.cos(), // undistort geometry on the hexagon
                            frequency * SQRT3B2 * y.sin(), // undistort geometry on the hexagon
                        ])
                })
                .iter()
                .sum::<f64>()
        };

        let x: f64 = TAU * datum.x;
        let y: f64 = TAU * (datum.x + datum.y); // undistort geometry on the hexagon
        fractional_brownian_motion(
            x + fractional_brownian_motion(x, y),
            y + fractional_brownian_motion(x, y),
        )
    };

    // prepare curve
    let elevation_curve: Spline<f64, f64> = {
        let step = 256f64.recip();
        Spline::from_vec(vec![
            Key::new(0., 0., Interpolation::Linear),
            Key::new(OCNLV - 0.08, OCNLV - 12.0 * step, Interpolation::Linear),
            Key::new(OCNLV, OCNLV, Interpolation::Linear),
            Key::new(OCNLV + 0.04, OCNLV + 8.0 * step, Interpolation::Linear),
            // Key::new(0.03, shelf + 4.0 * step, Interpolation::Linear),
            // Key::new(0.27, shelf + 16.0 * step, Interpolation::Linear),
            Key::new(1., 1., Interpolation::Linear),
        ])
    };

    elevation_base.operate_by_index(|jndex| {
        let base = elevation_base.grid[jndex];
        let noise = toroidal_sample(
            &DatumZa::enravel(jndex, elevation_base.resolution).cast(elevation_base.resolution),
        );
        Elevation::confine(
            elevation_curve
                .clamped_sample(base + noise * (base - OCNLV).abs().powf(0.72).max(0.06))
                .unwrap(),
        )
    })
}

pub fn ocean_tiles(elevation: &Brane<Elevation>, ocean: Elevation) -> Brane<bool> {
    elevation.operate_by_value_ref(|value| value < &ocean)
}

pub fn altitude_above_ocean_level(
    elevation: &Brane<Elevation>,
    ocean: Elevation,
) -> Brane<Elevation> {
    elevation.operate_by_value_ref(|value| {
        Elevation::confine((value.release() - ocean.release()).max(0.))
    })
}

/* # insolation */

fn insolation_at_datum(datum: DatumRe, solar_time: f64) -> f64 {
    let solar_ellipse = |time: f64| -> DatumRe {
        let eccentricity: f64 = 0.42;
        let radius_major: f64 = 0.36;
        let angle: f64 = -TAU * 3f64.recip();

        let linear = eccentricity * radius_major;
        let radius_minor = (radius_major.powi(2) - linear.powi(2)).sqrt();

        // velocities come from the vis viva equation at abfocal and peryfocal points
        let velo_max: f64 =
            ((radius_major + linear) * (radius_major - linear).recip() * radius_major.recip())
                .sqrt();
        let velo_min: f64 =
            ((radius_major - linear) * (radius_major + linear).recip() * radius_major.recip())
                .sqrt();

        // should approximate the changening speed of the orbiting body
        //    due to the constant areal velocity
        let position = TAU
            * (velo_max * time - (7. * velo_max + 8. * velo_min - 15.) * time.powi(2)
                + (18. * velo_max + 32. * velo_min - 50.) * time.powi(3)
                - (20. * velo_max + 40. * velo_min - 60.) * time.powi(4)
                + (8. * velo_max + 16. * velo_min - 24.) * time.powi(5));

        let focus = DatumRe::new(linear * angle.cos(), linear * angle.sin());
        let ellipse = focus
            - DatumRe::new(
                radius_major * angle.cos() * position.cos(),
                radius_major * angle.sin() * position.cos(),
            )
            + DatumRe::new(
                radius_minor * -angle.sin() * position.sin(),
                radius_minor * angle.cos() * position.sin(),
            );
        ellipse.uncentre()
    };

    // encodes the relationship between the ground distance between points
    //    and the received insolation
    let insolation_curve = |distance: f64| -> f64 { (TAU * 4f64.recip() * distance).cos() };

    insolation_curve(datum.distance(&solar_ellipse(solar_time)))
}

pub fn temperature_average(resolution: Resolution) -> Brane<Temperature> {
    trace!("calculating average insolation");

    let detail = 6usize.pow(3);
    Brane::<Temperature>::create_by_datum(resolution, |datum| {
        Temperature::confine(
            (0..detail)
                .map(|time| insolation_at_datum(datum, time as f64 / detail as f64))
                .sum::<f64>()
                / detail as f64,
        )
    })
}

pub fn temperature_at_ocean_level(
    solar_time: f64,
    temperature_average: &Brane<Temperature>,
    continentality: &Brane<f64>,
) -> Brane<Temperature> {
    let temperature_value = |insol: f64, insol_avg: f64, cont: f64| -> Temperature {
        Temperature::confine(insol_avg + 4. * cont * (insol - insol_avg))
    };

    match temperature_average.resolution == continentality.resolution {
        true => continentality.operate_by_index(|j| {
            temperature_value(
                insolation_at_datum(
                    DatumZa::enravel(j, continentality.resolution).cast(continentality.resolution),
                    solar_time,
                ),
                temperature_average.grid[j].release(),
                continentality.grid[j],
            )
        }),
        false => panic!(),
    }
}

pub fn temperature_at_altitude(
    temperature_at_ocean: &Brane<Temperature>,
    altitude_above_ocean: &Brane<Elevation>,
) -> Brane<Temperature> {
    let lapse_rate = 162f64.recip(); // fall in temperature for one meter
    let lapse_value = |altitude: Elevation| -> f64 { altitude.meters() as f64 * lapse_rate };
    temperature_at_ocean.operate_by_index(|j| {
        Temperature::from_celcius(
            temperature_at_ocean.grid[j].celcius() - lapse_value(altitude_above_ocean.grid[j]),
        )
    })
}

/* # continentality */

fn wind(
    elevation_source: Elevation,
    temperature_source: Temperature,
    elevation_target: Elevation,
    temperature_target: Temperature,
) -> f64 {
    // should return values between 0 and 2
    //    where 0 means no wind and 2 means lots wind
    ((temperature_source.release() - temperature_target.release())
        + 18f64.powf(
            ((elevation_source.release() - elevation_target.release()).min(0.)
                * TAU
                * 4f64.recip())
            .tan(),
        ))
    .max(0.)
}

pub fn continentality(
    altitude_above_ocean: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    ocean_tiles: &Brane<bool>,
) -> Brane<f64> {
    trace!("calculating continentality");
    let resolution = altitude_above_ocean.resolution;

    let cowind = |elevation_source: Elevation,
                  temperature_source: Temperature,
                  elevation_target: Elevation,
                  temperature_target: Temperature|
     -> f64 {
        2. - wind(
            elevation_source,
            temperature_source,
            elevation_target,
            temperature_target,
        )
    };

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
                + 8. * cowind(
                    altitude_above_ocean.grid[here_jndex],
                    temperature.grid[here_jndex],
                    altitude_above_ocean.grid[jndex],
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
    let curve = |x: f64| -> f64 { 2f64.powf(x) - 1. };
    temperature.operate_by_value_ref(|value| {
        Precipitation::confine(curve(value.release().max(0.)).max(216f64.recip()))
    })
}

/* # rainfall */

#[derive(Clone, Copy)]
pub struct Counted<T> {
    value: T,
    counter: usize,
}

impl<T> Counted<T> {
    pub fn new(value: T) -> Self {
        Self { value, counter: 2 }
    }

    pub fn unwrap(self) -> T {
        self.value
    }

    pub fn is_nonzero(&self) -> bool {
        self.counter > 0
    }

    pub fn update(&mut self, value: T) {
        if self.is_nonzero() {
            self.value = value;
            self.counter -= 1;
        }
    }
}

pub fn rainfall(
    altitude_above_ocean: &Brane<Elevation>,
    temperature: &Brane<Temperature>,
    evaporation_potential: &Brane<Precipitation>,
    continentality: &Brane<f64>,
    ocean_tiles: &Brane<bool>,
) -> Brane<Precipitation> {
    let resolution = altitude_above_ocean.resolution;

    // create storage and prepopulate
    let mut queue = VecDeque::new();
    let mut precipitation = evaporation_potential.operate_by_index(|jndex| {
        Counted::new(evaporation_potential.grid[jndex] * (1. - continentality.grid[jndex]))
    });

    // preopulate oceans with evaporation
    for jndex in 0..resolution.square() {
        if ocean_tiles.grid[jndex] {
            queue.push_back(DatumZa::enravel(jndex, resolution));
        }
    }

    while !queue.is_empty() {
        for target in queue
            .pop_front()
            .expect("queue was not empty")
            .ambit_toroidal(resolution.into())
        {
            let jndex_target = target.unravel(resolution);
            let rain = target
                .ball_toroidal(1, resolution.into())
                .iter()
                .map(|source| {
                    let jndex_source = source.unravel(resolution);
                    std::cmp::min_by(
                        evaporation_potential.grid[jndex_source],
                        precipitation.grid[jndex_source].unwrap(),
                        |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less), // in case of error return precipitation
                    ) * wind(
                        altitude_above_ocean.grid[jndex_source],
                        temperature.grid[jndex_source],
                        altitude_above_ocean.grid[jndex_target],
                        temperature.grid[jndex_target],
                    ) * (1.
                        + source
                            .ambit_toroidal(resolution.into())
                            .iter()
                            .map(|neighbour| {
                                let jndex_neighbour = neighbour.unravel(resolution);
                                wind(
                                    altitude_above_ocean.grid[jndex_source],
                                    temperature.grid[jndex_source],
                                    altitude_above_ocean.grid[jndex_neighbour],
                                    temperature.grid[jndex_neighbour],
                                )
                            })
                            .sum::<f64>())
                    .recip()
                })
                .reduce(|a, b| a + b)
                .unwrap_or_else(|| Precipitation::confine(0.));
            let old_rain = precipitation
                .grid
                .get_mut(jndex_target)
                .expect("index from brane");
            if old_rain.is_nonzero() {
                if old_rain.unwrap() < rain {
                    old_rain.update(rain);
                    queue.push_back(target);
                } else {
                    old_rain.update(rain * 0.7 + old_rain.unwrap() * 0.3);
                }
            }
        }
    }

    let adjustment_curve = |x: Precipitation| Precipitation::confine(2f64.powf(x.release()) - 1.);

    precipitation.operate_by_index(|jndex| adjustment_curve(precipitation.grid[jndex].unwrap()))
}

/* # watershed */

/// calculate the amount of water flowing down to every datum
pub fn shed(slope: &Flux<Elevation>, rainfall: &Brane<Precipitation>) -> Brane<Precipitation> {
    fn shed_at_node(
        node: NodeIndex,
        shed: &mut Brane<Precipitation>,
        slope: &Flux<Elevation>,
        rainfall: &Brane<Precipitation>,
    ) -> Precipitation {
        // this has to be a function, not a closure
        //    sincle closures cannot be called recursively
        let datum = slope.graph[node];
        let moisture = rainfall.grid[datum.unravel(slope.resolution)]
            + slope
                .graph
                .edges_directed(node, Direction::Incoming)
                .map(|edge| shed_at_node(edge.source(), shed, slope, rainfall))
                .reduce(|a, b| a + b)
                .unwrap_or_else(|| Precipitation::confine(0.));
        shed.grid[datum.unravel(slope.resolution)] = moisture;
        moisture
    }
    let mut shed = rainfall.operate_by_index(|_| Precipitation::confine(0.));
    for node in slope.roots.clone() {
        shed_at_node(node, &mut shed, slope, rainfall);
    }
    shed
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;
    const RES: Resolution = Resolution::confine(6);

    #[test]
    fn temperature_at_ocean_level_match() {
        let avg = temperature_average(RES);
        let brane_zero = temperature_at_ocean_level(
            0.,
            &avg,
            &Brane::<f64>::create_by_index(RES, |j| (j % 2) as f64),
        );
        let brane_half = temperature_at_ocean_level(
            0.5,
            &avg,
            &Brane::<f64>::create_by_index(RES, |j| (j % 2) as f64),
        );
        let brane_one = temperature_at_ocean_level(
            1.,
            &avg,
            &Brane::<f64>::create_by_index(RES, |j| (j % 2) as f64),
        );

        assert_float_eq!(
            avg.grid[0].release(),
            brane_zero.grid[0].release(),
            abs <= EPSILON
        );
        assert_float_eq!(
            avg.grid[0].release(),
            brane_half.grid[0].release(),
            abs <= EPSILON
        );
        assert_float_eq!(
            avg.grid[0].release(),
            brane_one.grid[0].release(),
            abs <= EPSILON
        );

        assert_float_ne!(
            avg.grid[1].release(),
            brane_zero.grid[1].release(),
            abs <= EPSILON
        );
        assert_float_ne!(
            avg.grid[1].release(),
            brane_half.grid[1].release(),
            abs <= EPSILON
        );

        assert_float_eq!(
            brane_zero.grid[1].release(),
            brane_one.grid[1].release(),
            abs <= EPSILON
        );
        assert_float_ne!(
            brane_zero.grid[1].release(),
            brane_half.grid[1].release(),
            abs <= EPSILON
        );
    }
}
