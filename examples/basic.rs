#![feature(try_from)]

use ffsvm::*;
use std::convert::TryFrom;

fn main() -> Result<(), Error> {
    let svm = DenseSVM::try_from(SAMPLE_MODEL)?;

    let mut problem = Problem::from(&svm);
    let features = problem.features();

    features[0] = 0.55838;
    features[1] = -0.157895;
    features[2] = 0.581292;
    features[3] = -0.221184;

    svm.predict_value(&mut problem)?;

    assert_eq!(problem.solution(), Solution::Label(12));

    Ok(())
}
