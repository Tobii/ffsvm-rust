// TODO:
// * Get away from "monolithic" impl's for a struct, towards small traits and *their* impls for a struct


#![feature(toowned_clone_into)]
#![feature(test)]
#![feature(repr_simd)]

extern crate faster;
extern crate itertools;
#[macro_use] extern crate nom;
extern crate rand;
extern crate rayon;
extern crate test;

mod vectors;
mod kernel;
mod parser;
mod svm;
mod random;
pub mod util;

pub use kernel::RbfKernel;
pub use svm::{SVM, Class, Problem, RbfCSVM, PredictProblem};
pub use parser::{ModelFile, FromModelFile};
pub use vectors::SimdOptimized;
pub use random::Randomize;
