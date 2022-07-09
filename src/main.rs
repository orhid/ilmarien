use ilmarien::carto::{colour as clr, render::Renderable};
use log::info;

#[allow(dead_code)]
fn run_once() {
    use ilmarien::climate::circulation::{ocean_tiles, OCNLV};
    use ilmarien::climate::cosmos::Cosmos;
    use ilmarien::units::{Elevation, Unit};
    let cosmos = Cosmos::load();

    ocean_tiles(&cosmos.altitude, Elevation::confine(OCNLV))
        .operate_by_value(|value| if value { u8::MIN } else { u8::MAX })
        .save_raw_low("land-tiles".to_string());
    /*
    cosmos.altitude.downgrade(3).render_triple(
        "elevation".to_string(),
        clr::TopographyInk::new(Elevation::confine(3f64.recip())),
    );
    */
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
    /*
    use ilmarien::climate::vegetation::Vege;
    let vege = cosmos.vege().operate_by_value(|value| match value {
        Some(zone) => match zone {
            Vege::Frost => 0.,
            Vege::Stone | Vege::Sand => 0.25,
            _ => 0.5,
        },
        None => 1.,
    });
    vege.save_f64("vege".to_string());
    // vege.render_triple("vege".to_string(), clr::KoppenInk);
    */
    info!("finished simulation");
}

fn main() {
    pretty_env_logger::init_timed();
    info!("initialising ilmarien");
    run_once();
    info!("computation completed")
}
