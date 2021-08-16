use crate::climate::{geology as glg, hydrology as hdr, radiation as rad, surface as srf};
//use ilmarien::imaging::{colour as clr, render::Renderable};

#[allow(dead_code)]
fn full_simulation(resolution: usize, seed: u32) {
    // generate an initial elevation model
    let elevation = glg::elevation_generate(resolution + seed as usize, seed);

    // TODO: rock types map

    //  calculate initial ocean levels
    let ocean = hdr::ocean_initialise(resolution, &elevation);

    //  calculate the surface types and their associated properties
    let surface_type = srf::surface_type_calculate(resolution, &ocean);
    let surface_level = srf::surface_level_calculate(resolution, &elevation, &ocean);

    // calculate temperature
    let insolation = rad::insolation_calculate(resolution / 3);
    let temperature = rad::temperature_calculate(resolution / 3, &insolation, &surface_type);

    // TODO: calculate surface pressure
    let pressure = rad::pressure_calculate(resolution / 3, &temperature);

    // TODO: simulate rainfall
    // evaporation should subtract from the ocean map
    // a single droplet should travel for some time and then drop to the surface
    // this should produce a rainfall map as a byproduct
    // falling droplet should erode the surface until it stops moving
    // then is should deposit its water into the ocean map
    // water levels should be diffused one in a while
    // this should include flooding low lying land

    // TODO: simulate glaciers
    // cut off some amount of heat
    // based on the rainfall map, simulate the accumulation of glaciers
}
