//!  Library to calculate next semantic version based on
//! [conventional commits](https://www.conventionalcommits.org/).
//!
//! It does not analyze git history, the list of commits must be provided by the user.
//!
//! # Version changes
//!
//! If conventional commits are not used, the patch is incremented.
//!
//! ```rust
//! use semver::Version;
//! use next_version::NextVersion;
//!
//! let commits = vec!["my change"];
//! assert_eq!(Version::new(1, 2, 3).next(commits), Version::new(1, 2, 4));
//! ```
//!
//! If a feature comment, is present and the major number is not 1,
//! than the minor is incremented
//!
//! ```rust
//! use semver::Version;
//! use next_version::NextVersion;
//!
//! let commits = vec!["my change", "feat: make coffe"];
//! assert_eq!(Version::new(1, 2, 4).next(&commits), Version::new(1, 3, 0));
//!
//! assert_eq!(Version::new(0, 2, 4).next(&commits), Version::new(0, 2, 5));
//! ```
//!
//! Breaking changes will increment:
//! - major if major is not 0.
//! - minor if major is 0.
//!
//! ```rust
//! use semver::Version;
//! use next_version::NextVersion;
//!
//! let commits = vec!["feat!: break user"];
//! assert_eq!(Version::new(1, 2, 4).next(&commits), Version::new(2, 0, 0));
//!
//! assert_eq!(Version::new(0, 4, 4).next(&commits), Version::new(0, 5, 0));
//! ```
//!
//! Pre-release versions are incremented in the same way, independently
//! by the type of commits:
//!
//! ```rust
//! use semver::Version;
//! use next_version::NextVersion;
//!
//! let commits = vec!["feat!: break user"];
//! let version = Version::parse("1.0.0-alpha.1.2").unwrap();
//! let expected = Version::parse("1.0.0-alpha.1.3").unwrap();
//! assert_eq!(version.next(commits.clone()), expected);
//!
//! // If the pre-release doesn't contain a version, `.1` is appended.
//! let version = Version::parse("1.0.0-beta").unwrap();
//! let expected = Version::parse("1.0.0-beta.1").unwrap();
//! assert_eq!(version.next(commits), expected);
//!
//! ```
//!
//! Build metadata isn't modified.
//!
//! ```rust
//! use semver::Version;
//! use next_version::NextVersion;
//!
//! let commits = vec!["my change"];
//! let version = Version::parse("1.0.0-beta.1+1.1.0").unwrap();
//! let expected = Version::parse("1.0.0-beta.2+1.1.0").unwrap();
//! assert_eq!(version.next(commits.clone()), expected);
//!
//! let version = Version::parse("1.0.0+abcd").unwrap();
//! let expected = Version::parse("1.0.1+abcd").unwrap();
//! assert_eq!(version.next(commits.clone()), expected);
//! ```

mod next_version;
mod version_increment;

pub use crate::{next_version::*, version_increment::*};
