use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

const DELIMITER: char = '.';
const PRE_RELEASE_DELIMITER: char = '-';
const BUILD_METADATA_DELIMITER: char = '+';

#[derive(Debug, Clone)]
pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidNumber(String),
    Empty,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            ParseError::InvalidNumber(msg) => write!(f, "Invalid number: {msg}"),
            ParseError::Empty => write!(f, "Empty version string"),
        }
    }
}

impl std::error::Error for ParseError {}

#[allow(dead_code)]
impl SemanticVersion {
    pub fn new(
        major: u64,
        minor: u64,
        patch: u64,
        pre_release: Option<String>,
        build_metadata: Option<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        }
    }

    pub fn none() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn parse(version_str: &str) -> Result<Self, ParseError> {
        if version_str.is_empty() {
            return Err(ParseError::Empty);
        }

        let cleaned = version_str.trim();
        let version_start = cleaned.find(|c: char| c.is_ascii_digit()).unwrap_or(0);
        let cleaned = &cleaned[version_start..];

        let (version_and_pre, build_metadata) =
            if let Some(pos) = cleaned.find(BUILD_METADATA_DELIMITER) {
                let (left, right) = cleaned.split_at(pos);
                (left, Some(right[1..].to_string()))
            } else {
                (cleaned, None)
            };

        let (version_core, pre_release) =
            if let Some(pos) = version_and_pre.find(PRE_RELEASE_DELIMITER) {
                let (left, right) = version_and_pre.split_at(pos);
                (left, Some(right[1..].to_string()))
            } else {
                (version_and_pre, None)
            };

        let parts: Vec<&str> = version_core.split(DELIMITER).collect();

        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat(format!(
                "Expected MAJOR.MINOR.PATCH, got: {version_core}"
            )));
        }

        let major = parts[0].parse::<u64>().map_err(|_| {
            ParseError::InvalidNumber(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse::<u64>().map_err(|_| {
            ParseError::InvalidNumber(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = parts[2].parse::<u64>().map_err(|_| {
            ParseError::InvalidNumber(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(Self::new(major, minor, patch, pre_release, build_metadata))
    }

    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    pub fn core_version(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        if self.major == 0 || other.major == 0 {
            self.major == other.major && self.minor == other.minor
        } else {
            self.major == other.major
        }
    }

    fn compare_pre_release(&self, other: &Self) -> Ordering {
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => {
                let a_parts: Vec<&str> = a.split('.').collect();
                let b_parts: Vec<&str> = b.split('.').collect();

                for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
                    match (a_part.parse::<u64>(), b_part.parse::<u64>()) {
                        (Ok(a_num), Ok(b_num)) => match a_num.cmp(&b_num) {
                            Ordering::Equal => continue,
                            other => return other,
                        },
                        (Ok(_), Err(_)) => return Ordering::Less,
                        (Err(_), Ok(_)) => return Ordering::Greater,
                        (Err(_), Err(_)) => match a_part.cmp(b_part) {
                            Ordering::Equal => continue,
                            other => return other,
                        },
                    }
                }

                a_parts.len().cmp(&b_parts.len())
            }
        }
    }
}

impl FromStr for SemanticVersion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(ref pre) = self.pre_release {
            write!(f, "-{pre}")?;
        }

        if let Some(ref build) = self.build_metadata {
            write!(f, "+{build}")?;
        }

        Ok(())
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SemanticVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre_release == other.pre_release
    }
}

impl Eq for SemanticVersion {}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            other => return other,
        }

        self.compare_pre_release(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let version = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build_metadata, None);
    }

    #[test]
    fn test_parsing_with_v_prefix() {
        let version = SemanticVersion::parse("v1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_parsing_with_version_prefix() {
        let version = SemanticVersion::parse("version-2.5.10").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 5);
        assert_eq!(version.patch, 10);
    }

    #[test]
    fn test_parsing_with_pre_release() {
        let version = SemanticVersion::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre_release, Some("alpha.1".to_string()));
        assert_eq!(version.build_metadata, None);
    }

    #[test]
    fn test_parsing_with_build_metadata() {
        let version = SemanticVersion::parse("1.0.0+build.123").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build_metadata, Some("build.123".to_string()));
    }

    #[test]
    fn test_parsing_full_version() {
        let version = SemanticVersion::parse("2.3.4-beta.2+build.456").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 3);
        assert_eq!(version.patch, 4);
        assert_eq!(version.pre_release, Some("beta.2".to_string()));
        assert_eq!(version.build_metadata, Some("build.456".to_string()));
    }

    #[test]
    fn test_parsing_complex_tag() {
        let version = SemanticVersion::parse("reelix-v0.34.1").unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 34);
        assert_eq!(version.patch, 1);
    }

    #[test]
    fn test_invalid_format() {
        assert!(SemanticVersion::parse("1.2").is_err());
        assert!(SemanticVersion::parse("1").is_err());
        assert!(SemanticVersion::parse("").is_err());
        assert!(SemanticVersion::parse("abc").is_err());
    }

    #[test]
    fn test_invalid_numbers() {
        assert!(SemanticVersion::parse("a.b.c").is_err());
        assert!(SemanticVersion::parse("1.x.3").is_err());
        assert!(SemanticVersion::parse("1.2.y").is_err());
    }

    #[test]
    fn test_display() {
        let v1 = SemanticVersion::new(1, 2, 3, None, None);
        assert_eq!(v1.to_string(), "1.2.3");

        let v2 = SemanticVersion::new(1, 0, 0, Some("alpha.1".to_string()), None);
        assert_eq!(v2.to_string(), "1.0.0-alpha.1");

        let v3 = SemanticVersion::new(
            2,
            3,
            4,
            Some("beta.2".to_string()),
            Some("build.123".to_string()),
        );
        assert_eq!(v3.to_string(), "2.3.4-beta.2+build.123");
    }

    #[test]
    fn test_comparison_basic() {
        let v1 = SemanticVersion::parse("1.0.0").unwrap();
        let v2 = SemanticVersion::parse("2.0.0").unwrap();
        assert!(v1 < v2);

        let v3 = SemanticVersion::parse("1.1.0").unwrap();
        let v4 = SemanticVersion::parse("1.2.0").unwrap();
        assert!(v3 < v4);

        let v5 = SemanticVersion::parse("1.1.1").unwrap();
        let v6 = SemanticVersion::parse("1.1.2").unwrap();
        assert!(v5 < v6);
    }

    #[test]
    fn test_comparison_equal() {
        let v1 = SemanticVersion::parse("1.2.3").unwrap();
        let v2 = SemanticVersion::parse("v1.2.3").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_comparison_with_pre_release() {
        let v1 = SemanticVersion::parse("1.0.0-alpha").unwrap();
        let v2 = SemanticVersion::parse("1.0.0").unwrap();
        assert!(v1 < v2);

        let v3 = SemanticVersion::parse("1.0.0-alpha").unwrap();
        let v4 = SemanticVersion::parse("1.0.0-beta").unwrap();
        assert!(v3 < v4);

        let v5 = SemanticVersion::parse("1.0.0-alpha.1").unwrap();
        let v6 = SemanticVersion::parse("1.0.0-alpha.2").unwrap();
        assert!(v5 < v6);
    }

    #[test]
    fn test_comparison_numeric_vs_alpha_prerelease() {
        let v1 = SemanticVersion::parse("1.0.0-1").unwrap();
        let v2 = SemanticVersion::parse("1.0.0-alpha").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn test_comparison_build_metadata_ignored() {
        let v1 = SemanticVersion::parse("1.0.0+build.1").unwrap();
        let v2 = SemanticVersion::parse("1.0.0+build.2").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_is_pre_release() {
        let v1 = SemanticVersion::parse("1.0.0").unwrap();
        assert!(!v1.is_pre_release());

        let v2 = SemanticVersion::parse("1.0.0-alpha").unwrap();
        assert!(v2.is_pre_release());
    }

    #[test]
    fn test_core_version() {
        let v1 = SemanticVersion::parse("1.2.3-alpha+build.123").unwrap();
        assert_eq!(v1.core_version(), "1.2.3");
    }

    #[test]
    fn test_compatibility() {
        let v1 = SemanticVersion::parse("1.2.3").unwrap();
        let v2 = SemanticVersion::parse("1.5.0").unwrap();
        let v3 = SemanticVersion::parse("2.0.0").unwrap();

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_compatibility_zero_versions() {
        let v1 = SemanticVersion::parse("0.1.0").unwrap();
        let v2 = SemanticVersion::parse("0.1.5").unwrap();
        let v3 = SemanticVersion::parse("0.2.0").unwrap();

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_from_str_trait() {
        let version: SemanticVersion = "1.2.3".parse().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_complex_pre_release_comparison() {
        let versions = vec![
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
        ];

        let parsed: Vec<SemanticVersion> = versions
            .iter()
            .map(|v| SemanticVersion::parse(v).unwrap())
            .collect();

        for i in 0..parsed.len() - 1 {
            assert!(
                parsed[i] < parsed[i + 1],
                "{} should be less than {}",
                parsed[i],
                parsed[i + 1]
            );
        }
    }

    #[test]
    fn test_real_world_version_comparison() {
        let current = SemanticVersion::parse("0.35.1").unwrap();
        let older = SemanticVersion::parse("reelix-v0.34.1").unwrap();
        let newer = SemanticVersion::parse("reelix-v0.36.0").unwrap();

        assert!(current > older);
        assert!(current < newer);
        assert!(older < newer);
    }
}
