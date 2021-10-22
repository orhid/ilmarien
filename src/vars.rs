/* # climate */

/* ## simulation */

// time of a single loop in the cosmic simulation
// actually captures both time and mass of a single cell
pub const TIME_LOCAL: f64 = 0.0032;

/* ## cosmos */

pub const INIT_OCEAN_LEVEL: f64 = 0.25; // initial ocean level
pub const ICE_COMP: f64 = 3.24; // snow to ice compression rate
pub const SOL_DEV: f64 = 0.12; // amplidute of solar deviation

/* ## geology */

pub const HEX_AREA: f64 = 260956870.632; // area of the entire world in square kilometers
pub const GEO_DETAIL: i32 = 12; // number of octaves in noise generation
pub const GEO_SCALE: f64 = 0.84; // scale for generated noise
pub const AMP_FACTOR: f64 = 1.44; // base for amplitude geometric series
pub const BLW_FACTOR: f64 = 1.68; // should blow results to [-1,1] range
pub const DST_FACTOR: f64 = 0.866025403784; // should slightly undistort terrain

/* ## radiation */

pub const SOL_DETAIL: i32 = 12; // radius of suns taken for insolation calculation
pub const SOL_POWER: f64 = 432.0; // power of solar radiation
pub const INIT_TEMP: f64 = -704.0; // initial world temperature
pub const MID_TEMP: f64 = 273.0; // initial world temperature

pub const INIT_PRES: f64 = 0.5; // initial surface pressure
pub const GAS_CONST: f64 = 144.0; // ideal gas constant
                                  // pub const LAPSE_CONST: f64 = -396.0; // pressure lapse
pub const LAPSE_RATE: f64 = 135.4752; // temperature lapse rate

/* ## hydrology */

pub const EVA_RATE: f64 = 0.0000152587890625; // amount of water evaporated at every cycle
pub const FLAT_RAIN: f64 = 0.18; // amount of moisture dropped while passing through flat terrain
