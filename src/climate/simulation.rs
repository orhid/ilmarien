#[allow(unused_imports)]
use crate::{
    carto::{colour as clr, render::Renderable},
    climate::{cosmos as csm, geology as glg, hydrology as hdr, radiation as rad},
    vars::*,
};

#[allow(unused_variables)]
fn single_loop(cosmos: &mut csm::Cosmos, resolution: usize, sol: f64) {
    // temperature
    let insolation = rad::insolation(resolution / 3, sol);
    let surface = cosmos.surface();
    let temperature = rad::temperature(&insolation, &surface);

    cosmos.solidify_snow();
    let icemelt = cosmos.form_glaciers(&temperature);

    // wind
    let pressure = rad::pressure(&temperature);
    let wind = rad::wind(&pressure);
    // TODO: move light sediment through wind

    // rainfall
    let evaporation = hdr::evaporation(&pressure, &surface, &temperature);
    // TODO: subract water from oceans
    let elevation = cosmos.elevation();
    let rainfall = hdr::rainfall(&elevation, &evaporation, &wind);
    // TODO: accumulate snow where cold rain falls

    let landflow = cosmos.landflow();
    // TODO: shed should accept a snowmelt layer
    // moreover, rainfall should be diminished by snow accumulation
    let shed = hdr::shed(&landflow, &rainfall);
    // TODO: move sediment through water flow
    // TODO: replenish oceans with collected water

    // TODO: solidify sediment
    cosmos.simplify_columns();
    cosmos.reflow_oceans();

    // TODO: simulate vegetation
    // rivers and rainfall combined with temperature can give some way to include vegetation
    // which should enable a change in evaporation
    // a couple of round of this could lead to more erosion than without vegetation

    cosmos.render(clr::TopographyInk::new(INIT_OCEAN_LEVEL));
}

#[allow(unused_variables)]
pub fn full_simulation(resolution: usize, seed: u32) {
    // initailise bedrock level
    let bedrock = glg::bedrock_level(resolution, seed);

    // TODO: rock types map

    //  initialise cosmic onion
    let mut cosmos = csm::Cosmos::initialise(&bedrock);

    for _ in 0..1 {
        single_loop(&mut cosmos, resolution, 1.0);
    }

    // TODO: simulate glaciers
    // cut off some amount of heat
    // run loop again,existing models should be able to produce glaciers
    // increase heat and run loop again, this should melt some glaciers

    // TODO: simulate seasons after the calamity, when the sun becomes unstable
    // this will lead to more diverse climate zones and ultimately better vegetation
    // this can also lead to food and resources maps and then to population and wealth maps
}
