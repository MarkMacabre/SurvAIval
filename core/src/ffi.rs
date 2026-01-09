use crate::search::{query_best_match, Dictionaries};
use crate::storage::Engine;
use crate::types::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

static mut DICTIONARIES: Option<Dictionaries> = None;

fn get_dictionaries() -> &'static Dictionaries {
    unsafe {
        if DICTIONARIES.is_none() {
            DICTIONARIES = Some(Dictionaries::load().expect("Failed to load dictionaries"));
        }
        DICTIONARIES.as_ref().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn surv_open(db_path: *const c_char, read_only: bool) -> *mut Engine {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(db_path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match Engine::open(Path::new(path_str), read_only) {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn surv_close(engine: *mut Engine) {
    if !engine.is_null() {
        unsafe {
            let _ = Box::from_raw(engine);
        }
    }
}

#[no_mangle]
pub extern "C" fn surv_query_best_match(
    engine: *mut Engine,
    query_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() || query_json.is_null() {
        return create_error_json("NullPointer", "Invalid input");
    }

    let engine = unsafe { &*engine };
    let c_str = unsafe { CStr::from_ptr(query_json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return create_error_json("InvalidUtf8", "Invalid UTF-8 in query"),
    };

    let context: QueryContext = match serde_json::from_str(json_str) {
        Ok(ctx) => ctx,
        Err(e) => return create_error_json("ParseError", &format!("Failed to parse JSON: {}", e)),
    };

    let dicts = get_dictionaries();

    match query_best_match(engine, &context, dicts) {
        Ok(best_match) => {
            let json = serde_json::to_string(&best_match).unwrap_or_else(|_| {
                r#"{"status":"error","error_code":"SerializationError","message":"Failed to serialize result"}"#.to_string()
            });
            match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => create_error_json("SerializationError", "Failed to create C string"),
            }
        }
        Err(e) => {
            let (code, message) = match e {
                EngineError::NoMatch => ("NoMatch", e.to_string()),
                EngineError::MissingUrgency => ("MissingUrgency", e.to_string()),
                EngineError::AmbiguousScenario(_) => ("AmbiguousScenario", e.to_string()),
                EngineError::AmbiguousEnvironment(_) => ("AmbiguousEnvironment", e.to_string()),
                _ => ("Error", e.to_string()),
            };
            create_error_json(code, &message)
        }
    }
}

#[no_mangle]
pub extern "C" fn surv_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

fn create_error_json(error_code: &str, message: &str) -> *mut c_char {
    let error = ErrorResponse {
        status: "error".to_string(),
        error_code: error_code.to_string(),
        message: message.to_string(),
    };
    let json = serde_json::to_string(&error).unwrap_or_else(|_| {
        r#"{"status":"error","error_code":"Unknown","message":"Unknown error"}"#.to_string()
    });
    CString::new(json).unwrap().into_raw()
}
