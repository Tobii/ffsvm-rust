use libc::{c_char, uint32_t};
use std::ffi::CStr;
use std::slice;
use std::ptr::{null_mut};
use std::convert::TryFrom;

use svm::{RbfCSVM, Problem, PredictProblem};
use parser::ModelFile;


/// Possible error conditions we can return.
enum Errors {
    Ok = 0,
    NullPointerPassed = -1,
    NoValidUTF8 = -2,
    ModelParseError = -20,
    SVMCreationError = -30,
    SVMNoModel = -31,
    SVMModelAlreadyLoaded = -32,
    ProblemPoolTooSmall = -40,
    ProblemLengthNotMultipleOfAttributes = -41,
    LabelLengthDoesNotEqualProblems = -42,
}


/// The main context we expose over FFI, containing everything
/// we need. 
pub struct Context {
    max_problems: usize,
    model: Option<Box<RbfCSVM>>,
    problems: Vec<Problem>,
}

#[no_mangle]
pub extern fn ffsvm_test(value: i32) -> i32 {
    println!("Function ffsvm_test({}); called. If you can read this, it works.", value);
    value * value
}


#[no_mangle]
pub extern fn ffsvm_context_create(context_ptr: *mut *mut Context) -> i32 {
    if context_ptr.is_null() { return Errors::NullPointerPassed as i32; }
    
    let context = Context {
        max_problems: 1,
        model: None,
        problems: Vec::new()
    };
    
    let boxed = Box::from(context);
    let context_raw = Box::into_raw(boxed);
    
    unsafe {
        *context_ptr = context_raw;
    }
    
    Errors::Ok as i32
}

#[no_mangle]
pub extern fn ffsvm_load_model(context_ptr: *mut Context, model_c_ptr: *const c_char) -> i32 {
    if context_ptr.is_null() { return Errors::NullPointerPassed as i32; }
    if model_c_ptr.is_null() { return Errors::NullPointerPassed as i32; }

    let context = unsafe { 
        &mut *context_ptr 
    };
    
    // Convert pointer to our strucutre  
    let c_str = unsafe {
        CStr::from_ptr(model_c_ptr)
    };

    // Create UTF8 string
    let model_str = match c_str.to_str() {
        Err(_) => { return Errors::NoValidUTF8 as i32; }
        Ok(s) => { s }
    };
    
    // Parse model
    let model = match ModelFile::try_from(model_str) {
        Err(_) => { return Errors::ModelParseError as i32; }
        Ok(m) => { m }
    };
    
    // Convert into SVM
    let svm = match RbfCSVM::try_from(&model) {
        Err(_) => { return Errors::SVMCreationError as i32; }
        Ok(m) => { m }
    };

    context.problems = (0 .. context.max_problems).map(|_| Problem::from(&svm)).collect();
    context.model = Some(Box::from(svm));

    Errors::Ok as i32
}




#[no_mangle]
pub extern fn ffsvm_set_max_problems(context_ptr: *mut Context, max_problems: u32) -> i32 {
    if context_ptr.is_null() { return Errors::NullPointerPassed as i32; }


    let context = unsafe {
        &mut *context_ptr
    };
    
    match &context.model {
        &None => { context.max_problems = max_problems as usize; }
        &Some(_) => { return Errors::SVMModelAlreadyLoaded as i32; }
    }

    Errors::Ok as i32
}



#[no_mangle]
pub extern fn ffsvm_predict_values(context_ptr: *mut Context, features_ptr: *mut f32, features_len: u32, labels_ptr: *mut u32, labels_len: u32) -> i32 {
    if context_ptr.is_null() { return Errors::NullPointerPassed as i32; }
    if features_ptr.is_null() { return Errors::NullPointerPassed as i32; }
    if labels_ptr.is_null() { return Errors::NullPointerPassed as i32; }

    
    let context = unsafe {
        &mut *context_ptr
    };
    
    let features = unsafe {
        slice::from_raw_parts(features_ptr, features_len as usize)
    };

    let labels = unsafe {
        slice::from_raw_parts_mut(labels_ptr, labels_len as usize)
    };
    
    let svm = match &context.model {
        &None => { return Errors::SVMNoModel as i32; }
        &Some(ref model) => { model.as_ref() }
    };
    
    // Make sure the pointers have the right length
    let num_problems = match features.len() % svm.num_attributes {
        0 => { features.len() / svm.num_attributes }
        _ => { return Errors::ProblemLengthNotMultipleOfAttributes as i32; }
    };
    
    if num_problems > context.max_problems {
        return Errors::ProblemPoolTooSmall as i32;
    }

    if num_problems != labels_len as usize {
        return Errors::LabelLengthDoesNotEqualProblems as i32;
    }

    let problems = &mut context.problems;
    let num_attributes = svm.num_attributes;
    
    // Copy features to respective problems 
    for i in 0 .. num_problems {
        let this_problem = &features[i*num_attributes .. (i+1)*num_attributes];
        
        // Internal problem length can be longer than given one due to SIMD alignment.  
        for p in 0 .. num_attributes {
            problems[i].features[p] = this_problem[p];    
        }
    }
    
    // Predict values for given slice of actually used real problems.
    svm.predict_values(&mut problems[0..num_problems]);
    
    // And store the results
    for i in 0 .. num_problems {
        labels[i] = problems[i].label
    }

    Errors::Ok as i32
}


#[no_mangle]
pub extern fn ffsvm_context_destroy(context_ptr: *mut *mut Context) -> i32 {
    if context_ptr.is_null() { return Errors::NullPointerPassed as i32; }

    let context = unsafe {
        Box::from_raw(*context_ptr)
    };
    
    Errors::Ok as i32
}


