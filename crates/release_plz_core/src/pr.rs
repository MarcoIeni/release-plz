use cargo_metadata::Package;
use chrono::SecondsFormat;

use crate::UpdateResult;

pub const BRANCH_PREFIX: &str = "release-plz/";

#[derive(Debug)]
pub struct Pr {
    pub base_branch: String,
    pub branch: String,
    pub title: String,
    pub body: String,
}

impl Pr {
    pub fn new(
        default_branch: &str,
        packages_to_update: &[(Package, UpdateResult)],
        project_contains_multiple_pub_packages: bool,
    ) -> Self {
        Self {
            branch: release_branch(),
            base_branch: default_branch.to_string(),
            title: pr_title(packages_to_update, project_contains_multiple_pub_packages),
            body: pr_body(packages_to_update),
        }
    }
}

fn release_branch() -> String {
    let now = chrono::offset::Utc::now();
    // Convert to a string of format "2018-01-26T18:30:09Z".
    let now = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    // ':' is not a valid character for a branch name.
    let now = now.replace(':', "-");
    format!("{BRANCH_PREFIX}{now}")
}

fn pr_title(
    packages_to_update: &[(Package, UpdateResult)],
    project_contains_multiple_pub_packages: bool,
) -> String {
    if packages_to_update.len() > 1 || !project_contains_multiple_pub_packages {
        "chore: release".to_string()
    } else {
        let (package, update) = &packages_to_update[0];
        format!("chore({}): release v{}", package.name, update.version)
    }
}

fn pr_body(packages_to_update: &[(Package, UpdateResult)]) -> String {
    let header = "## 🤖 New release";
    let updates: String = packages_to_update
        .iter()
        .map(|(package, update)| {
            if package.version != update.version {
                format!(
                    "\n* `{}`: {} -> {}",
                    package.name, package.version, update.version
                )
            } else {
                format!("\n* `{}`: {}", package.name, package.version)
            }
        })
        .collect();
    let footer =
        "---\nThis PR was generated with [release-plz](https://github.com/MarcoIeni/release-plz/).";
    format!("{header}{updates}\n{footer}")
}
