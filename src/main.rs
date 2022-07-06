use ilmarien::carto::{colour as clr, render::Renderable};
use log::info;

#[allow(dead_code)]
fn run_once() {
    use ilmarien::climate::cosmos::Cosmos;
    // use ilmarien::units::Unit;
    let cosmos = Cosmos::simulate();

    /*
    let thermo = cosmos
        .charts
        .operate_by_value_ref(|chart| chart.thermos().release())
        .downgrade(4);
    thermo.render("thermo".to_string(), clr::BiHueInk::new(0.04, 0.54, 0.92));
    let igro = cosmos
        .charts
        .operate_by_value_ref(|chart| chart.igros())
        .downgrade(4);
    igro.render("igro".to_string(), clr::BiHueInk::new(0.54, 0.12, 0.92));
    let parallaxo = cosmos
        .charts
        .operate_by_value_ref(|chart| chart.parallaxos())
        .downgrade(4);
    parallaxo.render(
        "parallaxo".to_string(),
        clr::BiHueInk::new(0.42, 0.08, 0.92),
    );
    */
    let vege = cosmos.vege().downgrade(6);
    vege.render("vege".to_string(), clr::KoppenInk);
    info!("finished simulation");
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    run_once();
    info!("computation completed")
}
