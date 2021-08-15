// this will not build due to scope and is just a placeholder
// as this code works, but is just not relevant

/* ## watchly */

/*

const MAX_WATCH: usize = 16;

fn encircle(watch: usize) -> f64 {
    TAU * watch as f64 / (1.0 * MAX_WATCH as f64)
}

fn insolation_watch_curve(value: f64) -> f64 {
    value - 0.5
}

fn insolation_watch_calculate_sun(
    point: &Coordinate<f64>,
    sun: &Coordinate<f64>,
    watch: usize,
) -> f64 {
    let sunward = vector_elevation(&sun, 1.0) - vector_elevation(&point, 0.0);
    let angle = sunward[1].atan2(sunward[0]);
    (angle * 3.0 + encircle(watch)).sin().max(0.0).powf(0.72)
        * sunward.norm().powi(-2)
        * sunward.dot(&Vector3::new(0.0, 0.0, 1.0))
        / sunward.norm()
}

fn insolation_watch_calculate_point(point: &Coordinate<f64>, watch: usize) -> f64 {
    let detail = 8;
    let suns = point.find().ball(detail);
    insolation_watch_curve(
        suns.iter()
            .map(|sun| Coordinate {
                x: sun.x as f64,
                y: sun.y as f64,
            })
            .map(|sun| insolation_watch_calculate_sun(&point, &sun, watch))
            .sum::<f64>(),
    )
}

/// calculate insolation – the amount of radiation reaching the surface over a single watch
pub fn insolation_watch_calculate(resolution: usize, watch: usize) -> Brane<f64> {
    info!("calculating insolation map at watch {}", watch);

    let mut brane = Brane::from(
        Brane::<f64>::vec_par_iter(resolution)
            .map(|point| insolation_watch_calculate_point(&point, watch))
            .collect::<Vec<f64>>(),
    );
    brane.variable = format!("insolation-{}", watch);
    brane
}

/*
