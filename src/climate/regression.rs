use crate::carto::{
    brane::Brane,
    datum::{DatumRe, DatumZa},
    honeycomb::HoneyCellToroidal,
};
use nalgebra::{DMatrix, DVector};
use std::f64::consts::TAU;

fn coordinates(altitude: &Brane<f64>) -> DMatrix<f64> {
    let resolution = altitude.resolution;
    let datums = (0..resolution.pow(2))
        .map(|jndex| DatumZa::enravel(jndex, resolution))
        .collect::<Vec<DatumZa>>();
    let radius = DVector::<f64>::from_iterator(
        resolution.pow(2),
        datums.iter().map(|datum| {
            datum.dist_toroidal(&DatumZa::new(0, 0), resolution as i32) as f64
                * (resolution as f64).recip()
        }),
    );
    let centres = datums
        .into_iter()
        .map(|datum| datum.cast(resolution))
        .collect::<Vec<DatumRe>>();
    let xcos = DVector::<f64>::from_iterator(
        resolution.pow(2),
        centres.iter().map(|datum| (datum.x * TAU).cos()),
    );
    let xsin = DVector::<f64>::from_iterator(
        resolution.pow(2),
        centres.iter().map(|datum| (datum.x * TAU).sin()),
    );
    let ycos = DVector::<f64>::from_iterator(
        resolution.pow(2),
        centres.iter().map(|datum| (datum.y * TAU).cos()),
    );
    let ysin = DVector::<f64>::from_iterator(
        resolution.pow(2),
        centres.iter().map(|datum| (datum.y * TAU).sin()),
    );

    DMatrix::from_columns(&[
        radius,
        xcos,
        xsin,
        ycos,
        ysin,
        DVector::<f64>::from_iterator(resolution.pow(2), altitude.grid.clone().into_iter()),
    ])
}

fn lin_reg(x_train: &DMatrix<f64>, y_train: &DVector<f64>, x_test: &DMatrix<f64>) -> DVector<f64> {
    let columns = x_train.shape().1;
    let qr = x_train
        .clone()
        .insert_column(columns, 1.0)
        .into_owned()
        .qr();
    let (q, r) = (qr.q().transpose(), qr.r());
    let coeff = r.try_inverse().unwrap() * &q * y_train;
    //println!("{}", coeff.clone());
    let mul = coeff.rows(0, columns);
    let intercept = coeff[(columns, 0)];

    (x_test * &mul).add_scalar(intercept)
}

pub fn predict_month(
    temp_smol: &Brane<f64>,
    rain_smol: &Brane<f64>,
    pevt_smol: &Brane<f64>,
    altitude_smol: &Brane<f64>,
    continentality_smol: &Brane<f64>,
    altitude: &Brane<f64>,
    continentality: &Brane<f64>,
) -> (Brane<f64>, Brane<f64>, Brane<f64>) {
    // # prepare coordinates
    let mut a_smol = coordinates(altitude_smol);
    let mut a = coordinates(altitude);
    let columns = a.shape().1;

    a_smol = a_smol.insert_column(columns, 0.0);
    a_smol.set_column(
        columns,
        &DVector::<f64>::from_iterator(
            altitude_smol.resolution.pow(2),
            continentality_smol.grid.clone().into_iter(),
        ),
    );
    a = a.insert_column(columns, 0.0);
    a.set_column(
        columns,
        &DVector::<f64>::from_iterator(
            altitude.resolution.pow(2),
            continentality.grid.clone().into_iter(),
        ),
    );

    // # predict temperature
    let temp_smol_dv = DVector::<f64>::from_iterator(
        altitude_smol.resolution.pow(2),
        temp_smol.grid.clone().into_iter(),
    );
    let temp_dv = lin_reg(&a_smol, &temp_smol_dv, &a);

    // # predict rain
    a_smol = a_smol.insert_column(columns + 1, 0.0);
    a = a.insert_column(columns + 1, 0.0);
    a_smol.set_column(columns + 1, &temp_smol_dv);
    a.set_column(columns + 1, &temp_dv);

    let rain_smol_dv = DVector::<f64>::from_iterator(
        altitude_smol.resolution.pow(2),
        rain_smol.grid.clone().into_iter(),
    );
    let rain_dv = lin_reg(&a_smol, &rain_smol_dv, &a);

    // # predict potential evapotranspiration
    let pevt_smol_dv = DVector::<f64>::from_iterator(
        altitude_smol.resolution.pow(2),
        pevt_smol.grid.clone().into_iter(),
    );
    let pevt_dv = lin_reg(&a_smol, &pevt_smol_dv, &a);

    // # return
    (
        Brane::from(
            temp_dv
                .into_iter()
                .map(|v| 1f64.min(0f64.max(*v)))
                .collect::<Vec<f64>>(),
        ),
        Brane::from(
            rain_dv
                .into_iter()
                .map(|v| 1f64.min(0f64.max(*v)))
                .collect::<Vec<f64>>(),
        ),
        Brane::from(
            pevt_dv
                .into_iter()
                .map(|v| 1f64.min(0f64.max(*v)))
                .collect::<Vec<f64>>(),
        ),
    )
}
