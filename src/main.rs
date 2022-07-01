#[allow(unused_imports)]
use ilmarien::{
    carto::{brane::Resolution, colour as clr, render::Renderable},
    climate::geology::ocean_level,
    units::Unit,
};
use log::info;

use std::thread;

#[allow(dead_code)]
fn gen_terrain(res: Resolution, seed: u32) {
    use ilmarien::climate::geology::bedrock_elevation;

    let altitude = bedrock_elevation(res, seed);
    let variable = format!("{}-alt", seed);
    // altitude.stats();
    // altitude.save(variable.clone());
    altitude.render(variable, clr::TopographyInk::new(ocean_level(&altitude)));
}

#[allow(dead_code)]
fn run_once(res: Resolution, seed: u32) {
    use ilmarien::climate::simulation::Cosmos;
    let cosmos = Cosmos::simulate(res, seed);

    let altitude = cosmos.altitude.clone();
    altitude.render(
        format!("{}-alt", seed),
        clr::TopographyInk::new(ocean_level(&cosmos.altitude)),
    );
    /*
    let aridity = cosmos.charts.operate_by_value_ref(|chart| chart.aridity());
    aridity.render(format!("{}-aridity", seed), clr::HueInk::new(0.12, 0.92));
    let vege = cosmos.vege();
    vege.render(format!("{}-vege", seed), clr::KoppenInk);
    */
    info!("finished simulation at {}", seed);
}

#[allow(dead_code)]
fn measure(res: Resolution) {
    use ilmarien::climate::geology::bedrock_elevation;

    let (j, k) = (3, 7);
    let olvs = (0..108)
        .map(|s| j * s + k)
        .map(|s| ocean_level(&bedrock_elevation(res, s)).release())
        .collect::<Vec<f64>>();
    println!("{}", olvs.iter().sum::<f64>() / olvs.len() as f64);
}

#[allow(dead_code)]
fn run_many() {
    let res: Resolution = Resolution::confine(216);

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    /*
    let (j, k) = (13, 17);
    let count = 36;
    for s in 0..count {
        let seed = j * s + k;
        // spin up another thread
        children.push(thread::spawn(move || {
            gen_terrain(res, seed);
        }));
    }
    */
    let seeds = [7, 30, 31, 46, 64, 69, 121, 193, 214, 381];
    for seed in seeds {
        // spin up another thread
        children.push(thread::spawn(move || {
            gen_terrain(res, seed);
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
    //gen_terrain(Resolution::confine(324), 0);
    //measure(Resolution::confine(324));
    run_many();
    info!("computation completed")
}
