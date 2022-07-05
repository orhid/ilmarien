// use ilmarien::carto::{colour as clr, render::Renderable};
use log::info;

/*
#[allow(dead_code)]
fn run_once() {
    use ilmarien::climate::cosmos::Cosmos;
    let cosmos = Cosmos::simulate();

    // let elevation = cosmos.altitude.downgrade(6);
    /*
    elevation.render(
        "elevation".to_string(),
        clr::TopographyInk::new(ocean_level(&elevation)),
    );
    */
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
    // aridity.stats_raw();
    // aridity.render("aridity".to_string(), clr::HueInk::new(0.12, 0.92));
    // let vege = cosmos.vege().downgrade(6);
    // vege.render("vege".to_string(), clr::KoppenInk);
    info!("finished simulation");
}
*/

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    // run_once();
    info!("computation completed")
}
