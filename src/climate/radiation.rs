use crate::{
    carto::{
        brane::{Brane, Resolution},
        datum::{DatumRe, DatumZa},
        honeycomb::Hexagon,
    },
    units::{Elevation, Temperature, Unit},
};
use log::trace;
use std::f64::consts::TAU;

/* # insolation */

pub fn crve(t: f64) -> DatumRe {
    //TODO: hide this into insolation, once we are not rendering this
    let eccentricity: f64 = 2f64.recip();
    let radius_major: f64 = 3f64.recip();
    let angle: f64 = -TAU * 6f64.recip();

    let linear = eccentricity * radius_major;
    let radius_minor = (radius_major.powi(2) - linear.powi(2)).sqrt();

    // velocities come from the vis viva equation at abfocal and peryfocal points
    let velo_max: f64 =
        ((radius_major + linear) * (radius_major - linear).recip() * radius_major.recip()).sqrt();
    let velo_min: f64 =
        ((radius_major - linear) * (radius_major + linear).recip() * radius_major.recip()).sqrt();

    // should approximate the changening speed of the orbiting body
    //    due to the constant areal velocity
    let time = TAU
        * (velo_max * t - (7. * velo_max + 8. * velo_min - 15.) * t.powi(2)
            + (18. * velo_max + 32. * velo_min - 50.) * t.powi(3)
            - (20. * velo_max + 40. * velo_min - 60.) * t.powi(4)
            + (8. * velo_max + 16. * velo_min - 24.) * t.powi(5));

    let focus = DatumRe::new(linear * angle.cos(), linear * angle.sin());
    let ellipse = focus
        + DatumRe::new(
            radius_major * angle.cos() * time.cos(),
            radius_major * angle.sin() * time.cos(),
        )
        + DatumRe::new(
            radius_minor * -angle.sin() * time.sin(),
            radius_minor * angle.cos() * time.sin(),
        );
    ellipse.uncentre()
}

fn insolation_at_datum(datum: DatumRe, solar_time: f64) -> f64 {
    let solar_ellipse = |time: f64| -> DatumRe { crve(time) };

    // encodes the relationship between the ground distance between points
    //    and the received insolation
    let insolation_curve = |distance: f64| -> f64 { 1. - (TAU * 4f64.recip() * distance) };

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
        Temperature::confine(insol_avg + 4. * cont * (insol_avg - insol))
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
    let lapse_rate = 144f64.recip(); // fall in temperature for one meter
    let lapse_value = |altitude: Elevation| -> f64 { altitude.meters() as f64 * lapse_rate };
    temperature_at_ocean.operate_by_index(|j| {
        Temperature::from_celcius(
            temperature_at_ocean.grid[j].celcius() - lapse_value(altitude_above_ocean.grid[j]),
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::{assert_float_eq, assert_float_ne};
    const EPSILON: f64 = 0.0000_01;
    const RES: Resolution = Resolution::confine(6);

    #[test]
    fn insolation_values() {
        let brane_zero =
            Brane::<f64>::create_by_datum(RES, |datum| insolation_at_datum(datum, 0.0));
        let brane_half =
            Brane::<f64>::create_by_datum(RES, |datum| insolation_at_datum(datum, 0.5));
        let brane_one = Brane::<f64>::create_by_datum(RES, |datum| insolation_at_datum(datum, 1.0));

        assert_float_eq!(brane_zero.grid[0], 0., abs <= EPSILON);
        assert_float_eq!(brane_zero.grid[8], 1., abs <= EPSILON);
        assert_float_eq!(brane_zero.grid[24], 2., abs <= EPSILON);

        assert_float_eq!(brane_zero.grid[0], brane_one.grid[0], abs <= EPSILON);
        assert_float_eq!(brane_zero.grid[8], brane_one.grid[8], abs <= EPSILON);
        assert_float_eq!(brane_zero.grid[24], brane_one.grid[24], abs <= EPSILON);

        assert_float_ne!(brane_half.grid[0], 0., abs <= EPSILON);
        assert_float_ne!(brane_half.grid[8], 1., abs <= EPSILON);
        assert_float_ne!(brane_half.grid[24], 2., abs <= EPSILON);
    }

    #[test]
    fn temperature_average_values() {
        let brane = temperature_average(RES);
        assert_float_eq!(brane.grid[0].release(), 0., abs <= EPSILON);
        assert_float_eq!(brane.grid[8].release(), 1., abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 2., abs <= EPSILON);
    }

    #[test]
    fn temperature_at_ocean_level_values() {
        let brane = temperature_at_ocean_level(
            0.,
            &temperature_average(RES),
            &Brane::<f64>::create_by_index(RES, |j| (j % 2) as f64),
        );
        assert_float_eq!(brane.grid[0].release(), 0., abs <= EPSILON);
        assert_float_eq!(brane.grid[8].release(), 1., abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 2., abs <= EPSILON);
    }

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

    #[test]
    fn temperature_lapse_values() {
        let brane = temperature_at_altitude(
            &Brane::new(vec![Temperature::confine(1.); 36], RES),
            &Brane::create_by_index(RES, |j| Elevation::confine(j as f64 / 36.)),
        );
        assert_float_eq!(brane.grid[0].release(), 1., abs <= EPSILON);
        assert_float_eq!(brane.grid[8].release(), 0.703704, abs <= EPSILON);
        assert_float_eq!(brane.grid[24].release(), 0.111111, abs <= EPSILON);
    }
}
