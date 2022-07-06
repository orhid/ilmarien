use crate::climate::chart::Zone;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Thermo {
    Hot,
    Warm,
    Brisk,
    Cold,
    Frigid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Igro {
    Jungle,
    Forest,
    Meadow,
    Desert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parallaxo {
    Barline,
    Balanced,
    Olivine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneType(Thermo, Igro, Parallaxo);

impl From<Zone> for ZoneType {
    fn from(zone: Zone) -> Self {
        let thermos = match zone.thermos.celcius() {
            t if (30.0..f64::INFINITY).contains(&t) => Thermo::Hot,
            t if (21.0..30.).contains(&t) => Thermo::Warm,
            t if (12.0..21.).contains(&t) => Thermo::Brisk,
            t if (-2.0..12.).contains(&t) => Thermo::Cold,
            _ => Thermo::Frigid,
        };
        let igros = match zone.igros {
            i if (-0.03..f64::INFINITY).contains(&i) => Igro::Jungle,
            i if (-0.24..-0.03).contains(&i) => Igro::Forest,
            i if (-0.48..-0.24).contains(&i) => Igro::Meadow,
            _ => Igro::Desert,
        };
        let parallaxos = match zone.parallaxos {
            p if (0.12..f64::INFINITY).contains(&p) => Parallaxo::Barline,
            p if (-0.06..0.12).contains(&p) => Parallaxo::Balanced,
            _ => Parallaxo::Olivine,
        };
        Self(thermos, igros, parallaxos)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/*
/// how many people can this zone sustain
pub fn habitability(cell: Option<Vege>) -> f64 {
    match cell {
        Some(vege) => match vege {
            Vege::Frost => 0.02,
            Vege::Stone => 0.01,
            Vege::Tundra => 0.12,
            Vege::Prairie => 0.78,
            Vege::Savanna => 0.48,
            Vege::Sand => 0.02,
            Vege::Shrub => 0.54,
            Vege::Taiga => 0.42,
            Vege::Coniferous => 0.84,
            Vege::Decideous => 1.00,
            Vege::Monsoon => 0.72,
            Vege::Broadleaf => 0.54,
        },
        None => 0.,
    }
}
*/

impl From<ZoneType> for Vege {
    fn from(zone: ZoneType) -> Self {
        match (zone.0, zone.1, zone.2) {
            (Thermo::Frigid, Igro::Desert, _) => Vege::Stone,
            (Thermo::Frigid, _, _) => Vege::Frost,

            (Thermo::Cold, Igro::Desert, _) => Vege::Stone,
            (Thermo::Cold, Igro::Meadow, _) => Vege::Tundra,
            (Thermo::Cold, Igro::Forest, _) => Vege::Coniferous,
            (Thermo::Cold, Igro::Jungle, _) => Vege::Taiga,

            (Thermo::Brisk, Igro::Desert, _) => Vege::Sand,
            (Thermo::Brisk, Igro::Meadow, Parallaxo::Barline) => Vege::Tundra,
            (Thermo::Brisk, Igro::Meadow, _) => Vege::Prairie,
            (Thermo::Brisk, Igro::Forest, Parallaxo::Barline) => Vege::Decideous,
            (Thermo::Brisk, Igro::Forest, _) => Vege::Coniferous,
            (Thermo::Brisk, Igro::Jungle, _) => Vege::Decideous,

            (Thermo::Warm, Igro::Desert, _) => Vege::Sand,
            (Thermo::Warm, Igro::Meadow, Parallaxo::Olivine) => Vege::Shrub,
            (Thermo::Warm, Igro::Meadow, _) => Vege::Prairie,
            (Thermo::Warm, Igro::Forest, _) => Vege::Decideous,
            (Thermo::Warm, Igro::Jungle, Parallaxo::Balanced) => Vege::Broadleaf,
            (Thermo::Warm, Igro::Jungle, _) => Vege::Monsoon,

            (Thermo::Hot, Igro::Desert, _) => Vege::Sand,
            (Thermo::Hot, Igro::Meadow, Parallaxo::Olivine) => Vege::Shrub,
            (Thermo::Hot, Igro::Meadow, _) => Vege::Savanna,
            (Thermo::Hot, Igro::Forest, Parallaxo::Olivine) => Vege::Shrub,
            (Thermo::Hot, Igro::Forest, _) => Vege::Decideous,
            (Thermo::Hot, Igro::Jungle, _) => Vege::Broadleaf,
        }
    }
}
