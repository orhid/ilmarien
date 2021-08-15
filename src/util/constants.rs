/* # climate */

/* ## simulation */

// time of a single loop in the cosmic simulation
// actually captures both time and mass of a single cell
pub const TIME_LOCAL: f64 = 0.0032;

/* ## geology */

pub const HEX_AREA: f64 = 260956870.632; // area of the entire world in square kilometers
pub const GEO_DETAIL: i32 = 12; // number of octaves in noise generation
pub const GEO_SCALE: f64 = 0.84; // scale for generated noise
pub const AMP_FACTOR: f64 = 1.44; // base for amplitude geometric series
pub const BLW_FACTOR: f64 = 1.68; // should blow results to [-1,1] range
pub const DST_FACTOR: f64 = 0.866025403784; // should slightly undistort terrain

/* ## radiation */

pub const SOL_DETAIL: i32 = 3; // radius of suns taken for insolation calculation
pub const SOL_POWER: f64 = 324.0; // power of solar radiation
pub const INIT_TEMP: f64 = -358.0; // initial world temperature
pub const MID_TEMP: f64 = 273.0; // initial world temperature

/* ## hydrology */
pub const INIT_OCEAN_LEVEL: f64 = 0.25;
