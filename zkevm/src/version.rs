/// Major version
pub const MAJOR: u32 = 0;
/// Minor version
pub const MINOR: u32 = 1;
/// Patch version
pub const PATCH: u32 = 5;

/// Export versions as string
pub fn as_string() -> String {
    format!("{}.{}.{}", MAJOR, MINOR, PATCH)
}

#[cfg(test)]
mod tests {
    use crate::version::{as_string, MAJOR, MINOR, PATCH};

    #[test]
    fn test_version_string() {
        let expected = "0.1.5";
        assert_eq!(MAJOR, 0, "wrong version");
        assert_eq!(MINOR, 1, "wrong version");
        assert_eq!(PATCH, 5, "wrong version");
        assert_eq!(as_string(), expected, "wrong version");
    }
}
