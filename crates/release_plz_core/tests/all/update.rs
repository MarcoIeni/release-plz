use cargo_metadata::Version;
use release_plz_core::read_package;

use crate::helpers::{comparison_test::ComparisonTest, user_mock};

#[test]
fn up_to_date_project_is_not_touched() {
    let comparison_test = ComparisonTest::new();

    comparison_test.run_update();

    // the update should have not changed anything
    assert!(comparison_test.are_projects_equal());
}

#[test]
fn version_is_updated_when_project_changed() {
    let comparison_test = ComparisonTest::new();
    let feature_message = "do awesome stuff";
    user_mock::add_feature(&comparison_test.local_project(), feature_message);

    comparison_test.run_update();

    // the update should have changed the version
    assert!(!comparison_test.are_projects_equal());

    let local_package = read_package(comparison_test.local_project()).unwrap();
    assert_eq!(local_package.version, Version::new(0, 1, 1));
    expect_test::expect![[r####"
        # Changelog
        All notable changes to this project will be documented in this file.

        The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
        and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

        ## [Unreleased]

        ## [0.1.1] - 1970-01-01

        ### Added
        - do awesome stuff
    "####]]
    .assert_eq(&comparison_test.local_project_changelog());
}
