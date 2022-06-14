#[allow(unused_imports)]
/*
use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::geology::ocean_level,
    units::Unit,
};
*/
use log::info;

// use std::thread;

/*
#[allow(dead_code)]
fn gen_terrain(res: usize, seed: u32) {
    use ilmarien::carto::brane::Brane;
    use ilmarien::climate::geology::bedrock_elevation;

    let mut altitude = bedrock_elevation(res, seed);
    altitude.variable = format!("{}-alt", seed);
    Brane::from(
        altitude
            .grid
            .iter()
            .map(|j| j.release())
            .collect::<Vec<f64>>(),
    )
    .stats();
    println!("ocean lv: {}", ocean_level(&altitude).release());
    altitude.render(clr::TopographyInk::new(ocean_level(&altitude)));
}
*/

/*
#[allow(dead_code)]
fn run_once(res: usize, seed: u32) {
    let cosmos = Cosmos::sim_new(res, seed);
    if cosmos.score() < 0.1 {
        info!("achieved score at {}, rendering...", seed);
        let mut altitude = cosmos.altitude.clone();
        altitude.variable = format!("{}-alt", seed);
        altitude.render(clr::TopographyInk::new(ocean(&cosmos.altitude)));
        let mut vege = cosmos.vege();
        vege.variable = format!("{}-vege", seed);
        vege.render(clr::KoppenInk);
    }
    info!("finished simulation at {}", seed);
}

#[allow(dead_code)]
fn run_many() {
    let res: usize = 216;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let (j, k) = (3, 7);
    let count = 18;
    for s in 0..count {
        let seed = j * s + k;
        // spin up another thread
        children.push(thread::spawn(move || {
            run_once(res, seed);
        }));
    }

    for child in children {
        // wait for the thread to finish
        let _ = child.join();
    }
}
*/

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    // gen_terrain(324, 0);
    info!("computation completed")
}
