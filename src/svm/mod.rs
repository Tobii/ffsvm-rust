mod class;
mod csvm;
mod problem;

pub use self::{class::Class, problem::Problem};

use crate::kernel::{Kernel, RbfKernel};
use crate::vectors::Triangular;

/// An RBF [SVM] which is the main (and currently only) type of SVM we support.
///
/// You can obtain an [RbfCSVM] with the help of a [ModelFile]. Each SVM implements
/// [PredictProblem] to actually classify data.
pub type RbfCSVM = SVM<RbfKernel>;

/// Possible error types when classifying with a [SVM].
#[derive(Debug)]
pub enum SVMError {
    /// This can be emitted when creating a [SVM] from a [ModelFile]. For models generated by
    /// libSVM's `svm-train`, the most common reason this occurs is skipping attributes.
    /// All attributes must be in sequential order 0, 1, 2, ..., n. If they are not, this
    /// error will be emitted. For more details see the documentation provided in [ModelFile].
    SvmAttributesUnordered {
        /// The index process that was not a direct successor of the previous index. Can be used for
        /// easier debugging the model file.
        index: u32,

        /// The value of the given index. Can be used for debugging in conjunction with `index`.
        value: f32,

        /// The last index processed. If everything were alright, then `index` should equal
        /// `last_index + 1`.
        last_index: u32,
    },

    /// This error can be emitted by [PredictProblem::predict_probability()] in case the model loaded by
    /// [ModelFile] was not trained with probability estimates (`svm-train -b 1`).
    ModelDoesNotSupportProbabilities,

    /// Can be emitted by [PredictProblem::predict_probability()] when predicting probabilities
    /// and the internal iteration limit was exceeded.
    MaxIterationsExceededPredictingProbabilities,
}

#[derive(Clone, Debug, Default)]
pub struct Probabilities {
    a: Triangular<f64>,

    b: Triangular<f64>,
}

/// Generic support vector machine, template for [RbfCSVM].
///
/// The SVM holds a kernel, class information and all other numerical data read from
/// the [ModelFile]. It implements [PredictProblem] to predict [Problem] instances.
///
/// # Creating a SVM
///
/// The only SVM currently implemented is the [RbfCSVM]. It can be constructed from a
/// [ModelFile] like this:
///
/// ```ignore
/// let svm = RbfCSVM::try_from(&model)!;
/// ```
///
#[derive(Clone, Debug, Default)]
pub struct SVM<T>
where
    T: Kernel,
{
    /// Total number of support vectors
    pub(crate) num_total_sv: usize,

    /// Number of attributes per support vector
    pub(crate) num_attributes: usize,

    pub(crate) rho: Triangular<f64>,

    pub(crate) probabilities: Option<Probabilities>,

    /// SVM specific data needed for classification
    pub(crate) kernel: T,

    /// All classes
    pub(crate) classes: Vec<Class>,
}

impl<T> SVM<T>
where
    T: Kernel,
{
    /// Finds the class index for a given label.
    ///
    /// # Description
    ///
    /// This method takes a `label` as defined in the libSVM training model
    /// and returns the internal `index` where this label resides. The index
    /// equals the [Problem]'s `.probabilities` index where that label's
    /// probability can be found.
    ///
    /// # Returns
    ///
    /// If the label was found its index returned in the [Option]. Otherwise `None`
    /// is returned.
    ///
    pub fn class_index_for_label(&self, label: u32) -> Option<usize> {
        for (i, class) in self.classes.iter().enumerate() {
            if class.label != label {
                continue;
            }

            return Some(i);
        }

        None
    }

    /// Returns the class label for a given index.
    ///
    /// # Description
    ///
    /// The inverse of [SVM::class_index_for_label], this function returns the class label
    /// associated with a certain internal index. The index equals the [Problem]'s
    /// `.probabilities` index where a label's probability can be found.
    ///
    /// # Returns
    ///
    /// If the index was found it is returned in the [Option]. Otherwise `None`
    /// is returned.
    pub fn class_label_for_index(&self, index: usize) -> Option<u32> {
        if index >= self.classes.len() {
            None
        } else {
            Some(self.classes[index].label)
        }
    }

    /// Returns number of attributes, reflecting the libSVM model.
    pub fn attributes(&self) -> usize {
        self.num_attributes
    }

    /// Returns number of classes, reflecting the libSVM model.
    pub fn classes(&self) -> usize {
        self.classes.len()
    }
}

/// Implemented by [SVM]s to predict a [Problem].
///
/// # Predicting a label
///
/// To predict a label, first make sure the [Problem] has all features set. Then calling
/// ```ignore
/// svm.predict_value(&mut problem)!
/// ```
/// will update the `.label` field to correspond to the class label with the highest likelihood.
///
/// # Predicting a label and obtaining probability estimates.
///
/// If the libSVM model was trained with probability estimates FFSVM can not only predict the
/// label, but it can also give information about the likelihood distribution of all classes.
/// This can be helpful if you want to consider alternatives.
///
/// Probabilities are estimated like this:
///
/// ```ignore
/// svm.predict_probability(&mut problem)!
/// ```
///
/// Predicting probabilities automatically predicts the best label. In addition `.probabilities`
/// will be updated accordingly. The class labels for each `.probabilities` entry can be obtained
/// by [SVM]'s `class_label_for_index` and `class_index_for_label` methods.
///
pub trait PredictProblem
where
    Self: Sync,
{
    /// Predict a single value for a [Problem].
    ///
    /// The problem needs to have all `.features` set. Once this method returns,
    /// the [Problem]'s field `.label` will be set.
    fn predict_value(&self, _: &mut Problem) -> Result<(), SVMError>;

    /// Predict a probability value for a problem.
    ///
    /// The problem needs to have all `.features` set. Once this method returns,
    /// both the [Problem]'s field `.label` will be set, and all `.probabilities` will
    /// be set accordingly.
    fn predict_probability(&self, _: &mut Problem) -> Result<(), SVMError>;
}
