use crate::climate::{cosmos as csm, geology as glg, hydrology as hdr, radiation as rad};
//use ilmarien::imaging::{colour as clr, render::Renderable};

#[allow(dead_code)]
#[allow(unused_variables)]
fn full_simulation(resolution: usize, seed: u32) {
    // initailise bedrock level
    let bedrock = glg::bedrock_level(resolution, seed);

    // TODO: rock types map

    //  initialise cosmic onion
    //  calculate elevation ad surface models
    let cosmos = csm::initialise(&bedrock);
    let elevation = csm::elevation(&cosmos);
    let surface = csm::surface(&cosmos);

    // calculate surface temperature adn pressure
    let insolation = rad::insolation(resolution / 3, 1.0);
    let temperature = rad::temperature(&insolation, &surface);
    let pressure = rad::pressure(&temperature);

    // simulate rainfall
    // TODO: evaporation currently does not reduce ocean levels
    let evaporation = hdr::evaporation(&pressure, &surface, &temperature);
    let pressure_flux = rad::pressure_flux(&pressure);
    let rainfall = hdr::rainfall(&elevation, &evaporation, &pressure_flux);

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
}
