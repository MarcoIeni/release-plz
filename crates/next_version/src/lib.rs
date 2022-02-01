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

mod next_version;
mod version_increment;

pub use crate::{next_version::*, version_increment::*};
