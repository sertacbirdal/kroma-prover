use halo2_proofs::halo2curves::{
    bn256::{Fq, G1Affine},
    serde::SerdeObject,
};
use log::{error, info};
use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
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

fn check_kzg_params_official(params_dir: &str, degree: usize) -> bool {
    let params_path = format!("{params_dir}/params{degree}");
    let f = File::open(params_path).unwrap();
    let reader = &mut BufReader::new(f);

    let mut k = [0u8; 4];
    reader.read_exact(&mut k[..]).unwrap();
    let k = u32::from_le_bytes(k);
    if degree != k as usize {
        log::info!("not match k, expected({degree}) but {k}");
    }

    let generator_from_file = <G1Affine as SerdeObject>::read_raw_unchecked(reader);
    if generator_from_file != G1Affine::generator() {
        return false;
    }

    let sg_from_file = <G1Affine as SerdeObject>::read_raw_unchecked(reader);
    sg_from_file.x
        == Fq::from_raw([
            0xac15e801f2b91e69,
            0xbb3d11e31115dafb,
            0x7f8fcae1abf6d2e4,
            0x269350b5ecd44c00,
        ])
        && sg_from_file.y
            == Fq::from_raw([
                0xe2d29ad22f98b08c,
                0xbfbb0d65b2ebe926,
                0xe686071693e4fa85,
                0x094818a234be895a,
            ])
}

pub fn panic_if_kzg_params_is_not_official(params_dir: &str) {
    if !check_kzg_params_official(params_dir, *DEGREE) {
        panic!(
            "The official kzg parameters should be used for degree {}.",
            *DEGREE
        )
    }
    if !check_kzg_params_official(params_dir, *AGG_DEGREE) {
        panic!(
            "The official kzg parameters should be used for degree {}.",
            *AGG_DEGREE
        )
    }
}

#[cfg(test)]
mod tests {
    use ctor::{ctor, dtor};
    use std::fs;

    use zkevm::{
        circuit::{AGG_DEGREE, DEGREE},
        utils::create_params,
    };

    use super::{
        is_cancun_trace, panic_if_kzg_params_is_not_official, panic_if_kzg_params_not_found,
    };

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

    #[test]
    #[should_panic]
    fn test_not_official_kzg_params() {
        panic_if_kzg_params_is_not_official(TEST_PARAMS_DIR);
    }

    #[ignore]
    #[test]
    // NOTE(dongchangYoo): the official params are needed for this test.
    fn test_official_kzg_params() {
        panic_if_kzg_params_is_not_official("../kzg_params");
    }
}
