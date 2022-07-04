#[allow(unused_imports)]
use ilmarien::{
    carto::{brane::Resolution, colour as clr, render::Renderable},
    climate::geology::ocean_level,
    units::Unit,
};
use log::info;

use std::thread;

#[allow(dead_code)]
fn gen_terrain(seed: u32) {
    use ilmarien::climate::geology::bedrock;

    let elevation = bedrock(seed);
    let variable = "elevation-noised".to_string();
    // elevation.stats();
    elevation.save(variable.clone());
    // let elevation = elevation.upscale(Resolution::confine(1080));
    // elevation.render(variable, clr::TopographyInk::new(ocean_level(&elevation)));
}

#[allow(dead_code)]
fn run_once() {
    use ilmarien::climate::simulation::Cosmos;
    let cosmos = Cosmos::simulate();

    let elevation = cosmos.altitude.downgrade(6);
    elevation.render(
        "elevation".to_string(),
        clr::TopographyInk::new(ocean_level(&elevation)),
    );
    /*
    elevation.render_triple(
        "elevation".to_string(),
        clr::TopographyInk::new(ocean_level(&elevation)),
    );
    */
    let aridity = cosmos
        .charts
        .operate_by_value_ref(|chart| chart.aridity())
        .downgrade(6);
    aridity.render("aridity".to_string(), clr::HueInk::new(0.12, 0.92));
    let vege = cosmos.vege().downgrade(6);
    vege.render("vege".to_string(), clr::KoppenInk);
    info!("finished simulation");
}

#[allow(dead_code)]
fn run_many() {
    // let res: Resolution = Resolution::confine(216);

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    /*
    let (j, k) = (279, 461);
    let count = 36;
    for s in 0..count {
        let seed = j * s + k;
        */
    for seed in [4088, 6320, 41069] {
        // spin up another thread
        children.push(thread::spawn(move || {
            gen_terrain(seed);
        }));
    }
    for child in children {
        // wait for the thread to finish
        let _ = child.join();
    }
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    //gen_terrain(41069);
    run_once();
    //measure(Resolution::confine(324));
    // run_many();
    // test();
    info!("computation completed")
}
