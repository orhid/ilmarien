use crate::climate::chart::Zone;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn water(&self) -> f64 {
        match self {
            Vege::Frost => 0.12,
            Vege::Stone => 0.0,
            Vege::Tundra => 0.18,
            Vege::Prairie => 0.18,
            Vege::Savanna => 0.18,
            Vege::Sand => 0.0,
            Vege::Shrub => 0.12,
            Vege::Taiga => 0.42,
            Vege::Coniferous => 0.54,
            Vege::Decideous => 0.84,
            Vege::Monsoon => 0.96,
            Vege::Broadleaf => 1.0,
        }
    }
}

impl From<&Vege> for Zone {
    fn from(vege: &Vege) -> Self {
        match vege {
            Vege::Stone => Self::new(0.0, 0.0, -3.0, 12.0),
            Vege::Frost => Self::new(0.54, 0.0, -6.0, -6.0),
            Vege::Tundra => Self::new(0.54, 0.0, -3.0, 12.0),
            Vege::Prairie => Self::new(0.36, -0.24, 3.0, 18.0),
            Vege::Savanna => Self::new(0.42, 0.0, 24.0, 36.0),
            Vege::Sand => Self::new(0.18, 0.0, 27.0, 36.0),
            Vege::Shrub => Self::new(0.54, -0.48, 12.0, 24.0),
            Vege::Taiga => Self::new(0.96, 0.24, 0.0, 12.0),
            Vege::Coniferous => Self::new(0.96, 0.12, 3.0, 18.0),
            Vege::Decideous => Self::new(0.96, 0.0, 12.0, 24.0),
            Vege::Monsoon => Self::new(0.84, 0.54, 18.0, 36.0),
            Vege::Broadleaf => Self::new(1.08, 0.0, 24.0, 24.0),
        }
    }
}
