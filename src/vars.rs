/* # climate */

/* ## simulation */

// time of a single loop in the cosmic simulation
// actually captures both time and mass of a single cell
//pub const TIME_LOCAL: f64 = 0.0032;

/* ## cosmos */

pub const RES_SMALL: usize = 144; // resolution for small scale computation

/* ## geology */

//pub const HEX_AREA: f64 = 260956870.632; // area of the entire world in square kilometers

/* ## hydrology */

pub const EVA_RATE: f64 = 0.0000152587890625; // amount of water evaporated at every cycle
pub const FLAT_RAIN: f64 = 0.12; // amount of moisture dropped while passing through flat terrain
pub const RAIN_RES: usize = 144; // base resolution for rainfall calculation
