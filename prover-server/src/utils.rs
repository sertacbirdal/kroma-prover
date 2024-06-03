use log::{error, info};
use std::{fmt::Display, path::Path};
use zkevm::circuit::{AGG_DEGREE, DEGREE};

pub static KROMA_MSG_HEADER: &str = "KROMA";

pub fn kroma_msg<S: AsRef<str> + Display>(msg: S) -> String {
    format!("[{KROMA_MSG_HEADER}] {msg}")
}

pub fn kroma_info<S: AsRef<str> + Display>(msg: S) {
    info!("{}", kroma_msg(msg))
}

pub fn kroma_err<S: AsRef<str> + Display>(msg: S) {
    error!("{}", kroma_msg(msg))
}

pub fn is_cancun_trace(trace_json: &String) -> bool {
    trace_json.contains("TSTORE") || trace_json.contains("TLOAD") || trace_json.contains("MCOPY")
}

fn check_kzg_param_exists(params_dir: &str, degree: usize) -> bool {
    let params_path = format!("{params_dir}/params{degree}");
    Path::new(&params_path).exists()
}

pub fn panic_if_kzg_params_not_found(params_dir: &str) {
    if !check_kzg_param_exists(params_dir, *AGG_DEGREE) {
        panic!("kzg params for degree {} not found", *AGG_DEGREE)
    }
    if !check_kzg_param_exists(params_dir, *DEGREE) {
        panic!("kzg params for degree {} not found", *DEGREE)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use ctor::{ctor, dtor};

    use zkevm::{circuit::{AGG_DEGREE, DEGREE}, utils::create_params};

    use super::{is_cancun_trace, panic_if_kzg_params_not_found};

    pub static TEST_PARAMS_DIR: &str = "../target/kzg_params";

    #[ctor]
    fn setup() {
        // Generate directory for kzg params.
        fs::create_dir_all(TEST_PARAMS_DIR).unwrap();
        // Generate empty params files for testing.
        let file_path = format!("{TEST_PARAMS_DIR}/params{}", *DEGREE);
        let _ = create_params(file_path.as_str(), 1).unwrap();
        let file_path = format!("{TEST_PARAMS_DIR}/params{}", *AGG_DEGREE);
        let _ = create_params(file_path.as_str(), 1).unwrap();
    }

    #[dtor]
    fn teardown() {
        fs::remove_dir_all(TEST_PARAMS_DIR).unwrap()
    }

    #[test]
    fn test_is_cancun_trace() {
        let trace_str = fs::read_to_string("../zkevm/tests/traces/kroma/push0.json").unwrap();
        assert!(!is_cancun_trace(&trace_str));
        let trace_str = fs::read_to_string("../zkevm/tests/traces/wrong/mcopy.json").unwrap();
        assert!(is_cancun_trace(&trace_str));
        let trace_str = fs::read_to_string("../zkevm/tests/traces/wrong/tstore.json").unwrap();
        assert!(is_cancun_trace(&trace_str));
        let trace_str = fs::read_to_string("../zkevm/tests/traces/wrong/tload.json").unwrap();
        assert!(is_cancun_trace(&trace_str));
    }

    #[test]
    #[should_panic]
    fn test_kzg_params_not_found() {
        let params_dir = "../target/kzg_params_wrong/";
        panic_if_kzg_params_not_found(params_dir);
    }

    #[test]
    fn test_kzg_params_found() {
        panic_if_kzg_params_not_found(TEST_PARAMS_DIR);
    }
}
