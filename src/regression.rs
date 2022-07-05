use crate::{
    carto::{
        brane::Brane,
        datum::{DatumRe, DatumZa},
        honeycomb::HoneyCellToroidal,
    },
    units::Elevation,
};
use nalgebra::{DMatrix, DVector};
use std::f64::consts::TAU;

fn coordinates(altitude: &Brane<Elevation>) -> DMatrix<f64> {
    let resolution = altitude.resolution;
    let datums = (0..resolution.square())
        .map(|jndex| DatumZa::enravel(jndex, resolution))
        .collect::<Vec<DatumZa>>();
    let radius = DVector::<f64>::from_iterator(
        resolution.square(),
        datums.iter().map(|datum| {
            datum.dist_toroidal(&DatumZa::new(0, 0), resolution.into()) as f64
                * (f64::from(resolution)).recip()
        }),
    );
    let centres = datums
        .into_iter()
        .map(|datum| datum.cast(resolution))
        .collect::<Vec<DatumRe>>();
    let xcos = DVector::<f64>::from_iterator(
        resolution.square(),
        centres.iter().map(|datum| (datum.x * TAU).cos()),
    );
    let xsin = DVector::<f64>::from_iterator(
        resolution.square(),
        centres.iter().map(|datum| (datum.x * TAU).sin()),
    );
    let ycos = DVector::<f64>::from_iterator(
        resolution.square(),
        centres.iter().map(|datum| (datum.y * TAU).cos()),
    );
    let ysin = DVector::<f64>::from_iterator(
        resolution.square(),
        centres.iter().map(|datum| (datum.y * TAU).sin()),
    );

    DMatrix::from_columns(&[
        radius,
        xcos,
        xsin,
        ycos,
        ysin,
        DVector::<f64>::from_iterator(resolution.square(), altitude.release().grid.into_iter()),
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
    let mul = coeff.rows(0, columns);
    let intercept = coeff[(columns, 0)];

    (x_test * mul).add_scalar(intercept)
}

pub fn predict_brane(
    brane_smol: &Brane<f64>,
    altitude_smol: &Brane<Elevation>,
    continentality_smol: &Brane<f64>,
    altitude: &Brane<Elevation>,
    continentality: &Brane<f64>,
) -> Brane<f64> {
    // # prepare coordinates
    let mut a_smol = coordinates(altitude_smol);
    let mut a = coordinates(altitude);
    let columns = a.shape().1;

    a_smol = a_smol.insert_column(columns, 0.0);
    a_smol.set_column(
        columns,
        &DVector::<f64>::from_iterator(
            altitude_smol.resolution.square(),
            continentality_smol.grid.clone().into_iter(),
        ),
    );
    a = a.insert_column(columns, 0.0);
    a.set_column(
        columns,
        &DVector::<f64>::from_iterator(
            altitude.resolution.square(),
            continentality.grid.clone().into_iter(),
        ),
    );

    // # predict
    let brane_smol_dv = DVector::<f64>::from_iterator(
        altitude_smol.resolution.square(),
        brane_smol.grid.clone().into_iter(),
    );
    let brane_dv = lin_reg(&a_smol, &brane_smol_dv, &a);

    Brane::new(
        brane_dv.into_iter().copied().collect::<Vec<f64>>(),
        altitude.resolution,
    )
}
