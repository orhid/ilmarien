#[allow(unused_imports)]
use ilmarien::{
    carto::{brane::Resolution, colour as clr, render::Renderable},
    climate::geology::ocean_level,
    units::Unit,
};
use log::info;

// use std::thread;

#[allow(dead_code)]
fn gen_terrain(res: Resolution, seed: u32) {
    use ilmarien::climate::geology::bedrock_elevation;

    let altitude = bedrock_elevation(res, seed);
    let variable = format!("{}-alt", seed);
    altitude.stats();
    altitude.save(variable.clone());
    altitude.render(variable, clr::TopographyInk::new(ocean_level(&altitude)));
}

#[allow(dead_code)]
fn insol(res: Resolution, seed: u32) {
    use ilmarien::climate::geology::bedrock_elevation;
    use ilmarien::climate::hydrology::{continentality, evapotranspiration_potential, rainfall};
    use ilmarien::climate::radiation::{
        temperature_at_elevation, temperature_at_ocean_level, temperature_average,
    };

    let altitude = bedrock_elevation(res, seed);
    /*
    altitude.render(
        "elev".to_string(),
        clr::TopographyInk::new(ocean_level(&altitude)),
    );
    */

    let avg = temperature_average(res);
    //avg.stats();
    //avg.render(format!("t"), clr::CelciusInk);

    let cont = continentality(&altitude, &avg, ocean_level(&altitude));
    // cont.stats_raw();
    // cont.render("cont".to_string(), clr::HueInk::new(0.0, 0.92));

    let t0 = temperature_at_ocean_level(0., &avg, &cont);
    //t0.stats();
    //t0.render(format!("t0"), clr::CelciusInk);

    /*
    let t1 = temperature_at_ocean_level(0.5, &avg, &cont);
    t1.stats();
    // t1.render(format!("t1"), clr::CelciusInk);
    */

    let t0e = temperature_at_elevation(&t0, &altitude, ocean_level(&altitude));
    //t0e.stats();
    //t0e.render(format!("t0e"), clr::CelciusInk);

    let evp = evapotranspiration_potential(&t0e);
    evp.stats();

    let rain = rainfall(&altitude, &t0e, &evp, ocean_level(&altitude));
    rain.stats();
    rain.release()
        .render("rain".to_string(), clr::HueInk::new(0.54, 0.92));
}

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
    //gen_terrain(Resolution::confine(324), 0);
    insol(Resolution::confine(216), 6);
    info!("computation completed")
}
