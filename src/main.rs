use ilmarien::geology as glg;
use ilmarien::imaging::colour as clr;
use ilmarien::imaging::render::Renderable;

fn main() {
    for seed in 0..1 {
        let elevation = glg::elevation_generate(144 + seed, seed as u32);
        elevation.save();
        elevation.render(clr::ElevationInk);
    }
}
