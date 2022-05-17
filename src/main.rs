#[allow(unused_imports)]
use ilmarien::{
    carto::{colour as clr, render::Renderable},
    climate::simulation as sim,
};
use log::info;
//use std::thread;

#[allow(dead_code)]
fn test() {
    let res: usize = 432;
    let (j, k) = (2u32.pow(12), 3u32.pow(9));
    for s in 0..1 {
        sim::simulate(res, j * s + k);
    }
}

/*
fn calculate_mdes() {
    use ilmarien::climate::geology::bedrock_level;
    let res: usize = 432;
    let (j, k) = (78615, 57815);
    let mdes = (0..72)
        .map(|seed| bedrock_level(res, k * seed + j).mde())
        .collect::<Vec<f64>>();
    dbg!(mdes.iter().sum::<f64>() / mdes.len() as f64);
*/

/*
#[allow(dead_code)]
fn test_sim() {
    let res: usize = 216;

    // make a vector to hold the children which are spawned
    let mut children = vec![];

    let begin = 837;
    let count = 1;
    for seed in begin..begin + count {
        // spin up another thread
        children.push(thread::spawn(move || {
            let mut cosmos = csm::Cosmos::new(&glg::bedrock_level(res, seed));
            cosmos.sim_climate(2, 12);
            let mut vege = cosmos.vege();
            vege.variable = format!("vege-{}", seed);
            vege.render(clr::KoppenInk);
            info!("finished simulation at {}", seed);
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
    test();
    info!("computation completed")
}
