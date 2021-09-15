use crate::{
    climate::{
        cosmos::{Fabric, Layer},
        koppen::Koppen,
    },
    vars::*,
};

/* # colour spaces */

pub struct HSB {
    // all values are in the [0,1] interval
    hue: f64,
    sat: f64,
    brt: f64,
}

impl HSB {
    fn new(hue: f64, sat: f64, brt: f64) -> Self {
        HSB { hue, sat, brt }
    }

    fn paint(&self) -> String {
        RGB::from(self).paint()
    }
}

impl From<&RGB> for HSB {
    fn from(rgb: &RGB) -> Self {
        let value = *[rgb.r, rgb.g, rgb.b].iter().max().unwrap();
        let chroma = (value - *[rgb.r, rgb.g, rgb.b].iter().min().unwrap()) as f64 / 256.0;

        HSB::new(
            if chroma == 0.0 {
                0.0
            } else if value == rgb.r {
                (rgb.g - rgb.b) as f64 / (6.0 * chroma) + if rgb.g <= rgb.b { 0.0 } else { 1.0 }
            } else if value == rgb.g {
                (rgb.b - rgb.r) as f64 / (6.0 * chroma) + 1.0 / 3.0
            } else {
                (rgb.r - rgb.g) as f64 / (6.0 * chroma) + 2.0 / 3.0
            },
            if value == 0 {
                0.0
            } else {
                chroma / value as f64
            },
            value as f64 / 256.0,
        )
    }
}

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn new(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }

    fn paint(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

impl From<&HSB> for RGB {
    fn from(hsb: &HSB) -> Self {
        fn hsb2u8(n: u8, hue: f64, sat: f64, brt: f64) -> u8 {
            let k: f64 = (n as f64 + hue * 6.0) % 6.0;
            (256.0 * brt * (1.0 - sat * 0.0_f64.max(1.0_f64.min(k.min(4.0 - k))))) as u8
        }

        let HSB { hue, sat, brt } = hsb;
        RGB::new(
            hsb2u8(5, *hue, *sat, *brt),
            hsb2u8(3, *hue, *sat, *brt),
            hsb2u8(1, *hue, *sat, *brt),
        )
    }
}

//will do this later
pub struct HEX {}

/* # inks */

pub trait Ink<T> {
    fn paint(&self, sample: T) -> String;
}

/* ## abstract inks */

/// will vary the brightness at constant hue and saturation
pub struct HueInk {
    hue: f64,
    brt: f64,
}

impl HueInk {
    pub fn new(hue: f64, brt: f64) -> Self {
        HueInk { hue, brt }
    }
}

impl Ink<f64> for HueInk {
    fn paint(&self, sample: f64) -> String {
        HSB::new(self.hue, sample, self.brt).paint()
    }
}

/* ## geographic inks */

pub struct TempInk;

impl Ink<f64> for TempInk {
    fn paint(&self, sample: f64) -> String {
        if sample > MID_TEMP {
            HueInk::new(0.02, 0.94).paint((sample - MID_TEMP) / 48.0)
        } else {
            HueInk::new(0.54, 0.94).paint((MID_TEMP - sample) / 12.0)
        }
    }
}

pub struct PresInk;

impl Ink<f64> for PresInk {
    fn paint(&self, sample: f64) -> String {
        let mid = 1.0;
        if sample > mid {
            HueInk::new(0.78, 0.94).paint((sample - mid) * 12.0)
        } else {
            HueInk::new(0.12, 0.94).paint((mid - sample) * 12.0)
        }
    }
}

pub struct ElevationInk;

impl Ink<f64> for ElevationInk {
    // one unit is around 54 meters
    fn paint(&self, sample: f64) -> String {
        let shore: u8 = 63;
        let elv = (sample * 255.0) as u8;
        if elv < shore - 16 {
            RGB::new(53, 89, 92).paint()
        } else if elv < shore - 8 {
            RGB::new(94, 138, 130).paint()
        } else if elv < shore - 2 {
            RGB::new(134, 163, 151).paint()
        } else if elv < shore {
            RGB::new(162, 184, 170).paint()
        } else if elv < shore + 2 {
            RGB::new(243, 245, 237).paint()
        } else if elv < shore + 4 {
            RGB::new(233, 235, 216).paint()
        } else if elv < shore + 8 {
            RGB::new(214, 213, 188).paint()
        } else if elv < shore + 16 {
            RGB::new(199, 191, 163).paint()
        } else if elv < shore + 32 {
            RGB::new(184, 165, 134).paint()
        } else if elv < shore + 64 {
            RGB::new(163, 131, 104).paint()
        } else if elv < shore + 128 {
            RGB::new(138, 95, 80).paint()
        } else {
            RGB::new(115, 71, 67).paint()
        }
    }
}

pub struct TopographyInk {
    ocean_level: f64,
}

impl TopographyInk {
    pub fn new(ocean_level: f64) -> Self {
        Self { ocean_level }
    }
}

impl Ink<Vec<Layer>> for TopographyInk {
    fn paint(&self, sample: Vec<Layer>) -> String {
        let elevation = sample.iter().map(|layer| layer.depth).sum::<f64>();
        let top = sample.last().unwrap();
        match top.fabric {
            Fabric::Water => {
                if top.depth < 2.0 / 256.0 {
                    RGB::new(162, 184, 170).paint()
                } else if top.depth < 8.0 / 256.0 {
                    RGB::new(134, 163, 151).paint()
                } else if top.depth < 16.0 / 256.0 {
                    RGB::new(94, 138, 130).paint()
                } else {
                    RGB::new(53, 89, 92).paint()
                }
            }
            Fabric::Snow | Fabric::Ice => RGB::new(
                (255.0 * elevation) as u8,
                (255.0 * elevation) as u8,
                (255.0 * elevation) as u8,
            )
            .paint(),
            _ => {
                if elevation < self.ocean_level {
                    RGB::new(223, 235, 217).paint()
                } else if elevation < self.ocean_level + 2.0 / 256.0 {
                    RGB::new(243, 245, 237).paint()
                } else if elevation < self.ocean_level + 4.0 / 256.0 {
                    RGB::new(233, 235, 216).paint()
                } else if elevation < self.ocean_level + 8.0 / 256.0 {
                    RGB::new(214, 213, 188).paint()
                } else if elevation < self.ocean_level + 16.0 / 256.0 {
                    RGB::new(199, 191, 163).paint()
                } else if elevation < self.ocean_level + 32.0 / 256.0 {
                    RGB::new(184, 165, 134).paint()
                } else if elevation < self.ocean_level + 64.0 / 256.0 {
                    RGB::new(163, 131, 104).paint()
                } else if elevation < self.ocean_level + 128.0 / 256.0 {
                    RGB::new(138, 95, 80).paint()
                } else {
                    RGB::new(115, 71, 67).paint()
                }
            }
        }
    }
}

pub struct KoppenInk;

impl Ink<Koppen> for KoppenInk {
    fn paint(&self, sample: Koppen) -> String {
        match sample {
            Koppen::Af => RGB::new(34, 70, 122).paint(),
            Koppen::Am => RGB::new(43, 94, 153).paint(),
            Koppen::As => RGB::new(51, 122, 184).paint(),
            Koppen::BWh => RGB::new(184, 104, 51).paint(),
            Koppen::BWc => RGB::new(184, 51, 65).paint(),
            Koppen::BSh => RGB::new(214, 145, 99).paint(),
            Koppen::BSc => RGB::new(214, 99, 110).paint(),
            Koppen::Cfa => RGB::new(184, 170, 51).paint(),
            Koppen::Cfc => RGB::new(214, 201, 86).paint(),
            Koppen::Csa => RGB::new(120, 153, 43).paint(),
            Koppen::Csc => RGB::new(151, 184, 73).paint(),
            Koppen::Cwa => RGB::new(52, 122, 34).paint(),
            Koppen::Cwc => RGB::new(80, 153, 61).paint(),
            Koppen::Dfa => RGB::new(43, 153, 109).paint(),
            Koppen::Dfc => RGB::new(73, 184, 140).paint(),
            Koppen::Dfd => RGB::new(111, 214, 173).paint(),
            Koppen::Dsa => RGB::new(98, 43, 153).paint(),
            Koppen::Dsc => RGB::new(129, 73, 184).paint(),
            Koppen::Dsd => RGB::new(163, 111, 214).paint(),
            Koppen::Dwa => RGB::new(122, 34, 87).paint(),
            Koppen::Dwc => RGB::new(153, 61, 116).paint(),
            Koppen::Dwd => RGB::new(184, 95, 148).paint(),
            Koppen::EF => RGB::new(245, 218, 215).paint(),
            Koppen::ET => RGB::new(235, 150, 159).paint(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_eq::assert_float_eq;
    const EPSILON: f64 = 0.0001;

    #[test]
    fn hsb2rgb() {
        assert_eq!(RGB::from(&HSB::new(0.0, 1.0, 1.0)).r, 255);
        assert_eq!(RGB::from(&HSB::new(1.0 / 3.0, 1.0, 1.0)).g, 255);
        assert_eq!(RGB::from(&HSB::new(2.0 / 3.0, 1.0, 1.0)).b, 255);
    }

    #[test]
    fn rgb2hsb() {
        assert_float_eq!(
            HSB::from(&RGB::new(255, 0, 0)).hue,
            0.0 / 3.0,
            abs <= EPSILON
        );
        assert_float_eq!(
            HSB::from(&RGB::new(0, 255, 0)).hue,
            1.0 / 3.0,
            abs <= EPSILON
        );
        assert_float_eq!(
            HSB::from(&RGB::new(0, 0, 255)).hue,
            2.0 / 3.0,
            abs <= EPSILON
        );
    }
}
