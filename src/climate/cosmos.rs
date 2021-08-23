use log::info;

/* # fabrics */

#[derive(PartialEq)]
pub enum Fabric {
    Water,
    Snow,
    Ice,
    Stone,
}

impl From<Fabric> for u8 {
    fn from(surface: Fabric) -> Self {
        match surface {
            Fabric::Water => 0,
            Fabric::Snow => 1,
            Fabric::Ice => 2,
            Fabric::Stone => 3,
        }
    }
}

impl From<u8> for Fabric {
    fn from(value: u8) -> Self {
        match value {
            0 => Fabric::Water,
            1 => Fabric::Snow,
            2 => Fabric::Ice,
            3 => Fabric::Stone,
            _ => panic!(),
        }
    }
}
