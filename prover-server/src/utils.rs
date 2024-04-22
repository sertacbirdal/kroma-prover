use log::{error, info};
use std::fmt::Display;

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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::is_cancun_trace;

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
}
