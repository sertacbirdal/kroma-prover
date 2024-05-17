use regex::Regex;

/// Major version
pub const MAJOR: u32 = 0;
/// Minor version
pub const MINOR: u32 = 1;
/// Patch version
pub const PATCH: u32 = 5;

/// Trace versions that are compatible with Prover.
pub const TRACE_VERSIONS: [&str; 3] = ["0.5.1", "0.5.2", "0.5.3"];

/// ZKEVM circuit versions that are compatible with Prover.
pub const ZKEVM_CIRCUIT_VERSIONS: [&str; 1] = ["0.2.0"];

/// Export versions as string
pub fn as_string() -> String {
    format!("{}.{}.{}", MAJOR, MINOR, PATCH)
}

// Return "0.5.1" when given strings like "0.5.1", "v0.5.1", and "0.5.1-unstable".
fn format_version(version_string: &str) -> String {
    let re = Regex::new(r"^(?:v)?(\d+)\.(\d+)\.(\d+)(?:-.+)?$").unwrap();
    match re.captures(version_string) {
        Some(caps) => {
            let major: u32 = caps
                .get(1)
                .expect("version parsing error")
                .as_str()
                .parse()
                .unwrap();
            let minor: u32 = caps
                .get(2)
                .expect("version parsing error")
                .as_str()
                .parse()
                .unwrap();
            let patch: u32 = caps
                .get(3)
                .expect("version parsing error")
                .as_str()
                .parse()
                .unwrap();
            format!("{}.{}.{}", major, minor, patch)
        }
        _ => panic!("trace version parsing error, version: {:?}", version_string),
    }
}

pub fn check_trace_version(trace_version_str: &str) -> bool {
    #[cfg(feature = "enable-mock-trace")]
    if trace_version_str == "TEST" {
        return true;
    }
    let formatted_version = format_version(trace_version_str);
    TRACE_VERSIONS.contains(&formatted_version.as_str())
}

pub fn panic_if_wrong_circuit_version() {
    let circuit_version = zkevm_circuits::version::as_string();
    if !ZKEVM_CIRCUIT_VERSIONS.contains(&circuit_version.as_str()) {
        panic!(
            "Supporting ZKEVM_CIRCUITS versions: {:?}, but actual: {:?}",
            ZKEVM_CIRCUIT_VERSIONS, circuit_version
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::version::{
        as_string, check_trace_version, panic_if_wrong_circuit_version, MAJOR, MINOR, PATCH,
    };

    #[test]
    fn test_version_string() {
        let expected = "0.1.5";
        assert_eq!(MAJOR, 0, "wrong version");
        assert_eq!(MINOR, 1, "wrong version");
        assert_eq!(PATCH, 5, "wrong version");
        assert_eq!(as_string(), expected, "wrong version");
    }

    #[test]
    fn test_check_trace_version() {
        let trace_ver = "0.5.1";
        assert!(check_trace_version(trace_ver));
        let trace_ver_prefix_v = "v0.5.1";
        assert!(check_trace_version(trace_ver_prefix_v));
        let trace_ver_suffix = "v0.5.1-unstable";
        assert!(check_trace_version(trace_ver_suffix));

        let trace_ver_wrong = "v0.1.0";
        assert!(!check_trace_version(trace_ver_wrong));
    }

    #[test]
    fn test_check_circuit_version() {
        panic_if_wrong_circuit_version();
    }
}
