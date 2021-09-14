#[allow(unused_imports)]
use crate::{
    carto::{colour as clr, render::Renderable},
    climate::{cosmos as csm, geology as glg, hydrology as hdr, radiation as rad},
    vars::*,
};

#[allow(unused_variables)]
fn single_loop(cosmos: &mut csm::Cosmos, resolution: usize, sol: f64) {
    // temperature
    let surface = cosmos.surface();
    let temperature = rad::temperature(
        &rad::insolation(if resolution > 216 { 144 } else { 72 }, sol),
        &surface,
    );

    cosmos.solidify_snow();
    let icemelt = cosmos.form_glaciers(&temperature);

    // wind
    let pressure = rad::pressure(&temperature);
    let wind = rad::wind(&pressure);
    // TODO: move light sediment through wind

    // rainfall
    let evaporation = hdr::evaporation(&pressure, &surface, &temperature);
    cosmos.evaporate_oceans(&evaporation);
    let elevation = cosmos.elevation();
    let mut rainfall = hdr::rainfall(&elevation, &evaporation, &wind);
    cosmos.snowfall(&mut rainfall, &temperature);

    let landflow = cosmos.landflow();
    rainfall = rainfall + icemelt;
    let shed = hdr::shed(&landflow, &rainfall);
    cosmos.replenish_oceans(&landflow, &shed);
    // TODO: move sediment through water flow

    // TODO: solidify sediment
    cosmos.reflow_oceans();

    // TODO: simulate vegetation
    // rivers and rainfall combined with temperature can give some way to include vegetation
    // which should enable a change in evaporation
    // a couple of round of this could lead to more erosion than without vegetation
}

#[allow(unused_variables)]
pub fn full_simulation(resolution: usize, seed: u32) {
    // initailise bedrock level
    let bedrock = glg::bedrock_level(resolution, seed);

    // TODO: rock types map

    //  initialise cosmic onion
    let mut cosmos = csm::Cosmos::initialise(&bedrock);
    cosmos.render(clr::TopographyInk::new(INIT_OCEAN_LEVEL));

    for j in 0..6 {
        single_loop(&mut cosmos, resolution, 1.0);
    }
    cosmos.variable = "cosmos-after".to_string();
    cosmos.render(clr::TopographyInk::new(INIT_OCEAN_LEVEL));

    // TODO: simulate glaciers
    // cut off some amount of heat
    // run loop again,existing models should be able to produce glaciers
    // increase heat and run loop again, this should melt some glaciers

    // TODO: simulate seasons after the calamity, when the sun becomes unstable
    // this will lead to more diverse climate zones and ultimately better vegetation
    // this can also lead to food and resources maps and then to population and wealth maps
}
