use ilmarien::geology as glg;
//use ilmarien::imaging::cartography as crt;
use ilmarien::imaging::colour as clr;
use ilmarien::imaging::render::Renderable;

fn main() {
    for seed in 0..1 {
        let elevation = glg::elevation_generate(432 + seed, seed as u32);
        //let elevation = crt::load("elevation-432".to_string());
        //elevation.render(clr::HueInk::new(0.5, 0.0));
        elevation.save();
        elevation.render(clr::ElevationInk);
    }
}
