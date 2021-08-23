use crate::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
//use ilmarien::imaging::{colour as clr, render::Renderable};

#[allow(dead_code)]
fn full_simulation(resolution: usize, seed: u32) {
    /*
    // generate an initial elevation model
    let elevation = glg::bedrock_level(resolution + seed as usize, seed);

    // TODO: rock types map

    //  calculate initial ocean levels
    let ocean = hdr::ocean_initialise(resolution, &elevation);

    //  calculate the surface types and their associated properties
    let surface_type = srf::surface_type_calculate(resolution, &ocean);
    let surface_level = srf::surface_level_calculate(resolution, &elevation, &ocean);

    // calculate temperature
    let insolation = rad::insolation_calculate(resolution / 3);
    let temperature = rad::temperature_calculate(resolution / 3, &insolation, &surface_type);

    // calculate surface pressure
    let pressure = rad::pressure_calculate(resolution / 3, &temperature);

    // simulate rainfall
    // TODO: evaporation currently does not reduce ocean levels
    let evaporation =
        hdr::evaporation_calculate(resolution / 3, &surface_type, &temperature, &pressure);
    let rainfall = hdr::rainfall(&pressure, &evaporation, &surface_level);

    // TODO: simulate rivers
    // this should include sedimant transportation
    // as well as possibly changing ocean levels

    // TODO: simulate vegetation
    // rivers and rainfall combined with temperature can give some way to include vegetation
    // which should enable a change in evaporation
    // a couple of round of this could lead to more erosion than without vegetation

    // TODO: simulate glaciers
    // cut off some amount of heat
    // based on the rainfall map, simulate the accumulation of glaciers
    // then simulate them being melted

    // TODO: simulate seasons after the calamity, when the sun becomes unstable
    // this will lead to more diverse climate zones and ultimately better vegetation
    // this can also lead to food and resources maps and then to population and wealth maps
    */
}
