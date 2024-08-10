use crate::PackagesUpdate;
use chrono::SecondsFormat;

pub const BRANCH_PREFIX: &str = "release-plz-";
pub const OLD_BRANCH_PREFIX: &str = "release-plz/";

#[derive(Debug)]
pub struct Pr {
    pub base_branch: String,
    pub branch: String,
    pub title: String,
    pub body: String,
    pub draft: bool,
    pub labels: Vec<String>,
}

impl Pr {
    pub fn new(
        default_branch: &str,
        packages_to_update: &PackagesUpdate,
        project_contains_multiple_pub_packages: bool,
    ) -> Self {
        Self {
            branch: release_branch(),
            base_branch: default_branch.to_string(),
            title: pr_title(packages_to_update, project_contains_multiple_pub_packages),
            body: pr_body(packages_to_update, project_contains_multiple_pub_packages),
            draft: false,
            labels: vec![],
        }
    }

    pub fn mark_as_draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
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
    packages_to_update: &PackagesUpdate,
    project_contains_multiple_pub_packages: bool,
) -> String {
    let updates = packages_to_update.updates();
    let first_version = &updates[0].1.version;

    let are_all_versions_equal = || {
        updates
            .iter()
            .all(|(_, update)| &update.version == first_version)
    };

    if updates.len() == 1 && project_contains_multiple_pub_packages {
        let (package, _) = &updates[0];
        // The project is a workspace with multiple public packages and we are only updating one of them.
        // Specify which package is being updated in the PR title.
        format!("chore({}): release v{}", package.name, first_version)
    } else if updates.len() > 1 && !are_all_versions_equal() {
        // We are updating multiple packages with different versions, so we don't specify the version in the PR title.
        "chore: release".to_string()
    } else {
        // We are updating either:
        // - a single package without other public packages
        // - multiple packages with the same version.
        // In both cases, we can specify the version in the PR title.
        format!("chore: release v{first_version}")
    }
}

fn pr_body(
    packages_to_update: &PackagesUpdate,
    project_contains_multiple_pub_packages: bool,
) -> String {
    let header = "## 🤖 New release";

    let summary = packages_to_update.summary();
    let changes = {
        let changes = packages_to_update.changes(project_contains_multiple_pub_packages);
        format!(
            "<details><summary><i><b>Changelog</b></i></summary><p>\n\n{changes}\n</p></details>\n"
        )
    };

    let footer =
        "---\nThis PR was generated with [release-plz](https://github.com/MarcoIeni/release-plz/).";
    format!("{header}{summary}\n{changes}\n{footer}")
}
