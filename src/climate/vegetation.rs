use crate::climate::chart::Zone;
//use ord_subset::OrdSubsetIterExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Vege {
    Stone,
    Frost,
    Tundra,
    Prairie,
    Savanna,
    Sand,
    Shrub,
    Taiga,
    Coniferous,
    Decideous,
    Monsoon,
    Broadleaf,
}

impl Vege {
    pub fn array() -> Vec<Self> {
        vec![
            Vege::Stone,
            Vege::Frost,
            Vege::Tundra,
            Vege::Prairie,
            Vege::Savanna,
            Vege::Sand,
            Vege::Shrub,
            Vege::Taiga,
            Vege::Coniferous,
            Vege::Decideous,
            Vege::Monsoon,
            Vege::Broadleaf,
        ]
    }
}

/// amount of water that could be evaporated
pub fn hydro_potential(cell: Option<Vege>) -> f64 {
    match cell {
        Some(vege) => match vege {
            Vege::Frost => 0.12,
            Vege::Stone => 0.02,
            Vege::Tundra => 0.18,
            Vege::Prairie => 0.18,
            Vege::Savanna => 0.18,
            Vege::Sand => 0.01,
            Vege::Shrub => 0.12,
            Vege::Taiga => 0.42,
            Vege::Coniferous => 0.54,
            Vege::Decideous => 0.84,
            Vege::Monsoon => 0.96,
            Vege::Broadleaf => 1.0,
        },
        None => 0.84,
    }
}

const ARID_FACTOR: f64 = 0.36;

impl From<&Vege> for Zone {
    fn from(vege: &Vege) -> Self {
        match vege {
            // high aridity should correspond to arid areas, not wet DUMBASS
            Vege::Stone => Self::new(1.08 * ARID_FACTOR, 0.0, -3.0, 12.0),
            Vege::Sand => Self::new(0.96 * ARID_FACTOR, 0.0, 27.0, 36.0),

            Vege::Frost => Self::new(0.42 * ARID_FACTOR, 0.0, -6.0, -6.0),
            Vege::Tundra => Self::new(0.54 * ARID_FACTOR, 0.0, -3.0, 12.0),
            Vege::Prairie => Self::new(0.72 * ARID_FACTOR, -0.24, 3.0, 18.0),
            Vege::Shrub => Self::new(0.54 * ARID_FACTOR, -0.48, 12.0, 24.0),
            Vege::Savanna => Self::new(0.42 * ARID_FACTOR, 0.0, 24.0, 36.0),

            Vege::Monsoon => Self::new(0.24 * ARID_FACTOR, 0.54, 18.0, 36.0),
            Vege::Taiga => Self::new(0.12 * ARID_FACTOR, 0.24, 0.0, 12.0),
            Vege::Coniferous => Self::new(0.12 * ARID_FACTOR, 0.12, 3.0, 18.0),
            Vege::Decideous => Self::new(0.12 * ARID_FACTOR, 0.0, 12.0, 24.0),
            Vege::Broadleaf => Self::new(0.0 * ARID_FACTOR, 0.0, 24.0, 24.0),
        }
    }
}

impl From<&Zone> for Vege {
    fn from(zone: &Zone) -> Self {
        if zone.is_nan() {
            Self::Stone
        } else {
            match zone.aridity {
                a if a < 0.64 * ARID_FACTOR => {
                    if zone.tmax < 0.0 {
                        Vege::Frost
                    } else if zone.tmin > 18.0 {
                        Vege::Broadleaf // rainforest
                    } else if 3.0 * zone.tmin + zone.tmax - 12.0 * zone.swing - 36.0 < 0.0 {
                        Vege::Taiga // coniferous but wetter
                    } else {
                        Vege::Monsoon //decidous but wetter
                    }
                }
                a if a < 0.96 * ARID_FACTOR => {
                    if zone.tmax < 0.0 {
                        Vege::Frost
                    } else if zone.tmin > 24.0 {
                        Vege::Shrub
                    } else if 9.0 * zone.tmin + zone.tmax - 36.0 * zone.swing - 60.0 < 0.0 {
                        Vege::Coniferous
                    } else {
                        Vege::Decideous
                    }
                }
                a if a < 1.44 * ARID_FACTOR => {
                    if zone.tmax < 0.0 {
                        Vege::Frost
                    } else if zone.tmin > 18.0 {
                        Vege::Savanna
                    } else if zone.tmin + zone.tmax - 6.0 * zone.swing - 30.0 > 0.0 {
                        Vege::Shrub
                    } else if 5.0 * zone.tmin + 3.0 * zone.tmax + 24.0 * zone.swing - 24.0 < 0.0 {
                        Vege::Tundra
                    } else {
                        Vege::Prairie
                    }
                }
                _ => {
                    if zone.tmin > 0.0 {
                        Vege::Sand
                    } else {
                        Vege::Stone
                    }
                }
            }
        }
    }
}
