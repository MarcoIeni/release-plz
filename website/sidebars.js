/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    "intro",
    {
      type: "category",
      label: "CLI Usage",
      collapsed: false,
      link: { type: "doc", id: "usage/index" },
      items: [
        "usage/installation",
        "usage/update",
        "usage/release-pr",
        "usage/release",
        "usage/shell-completion",
        "usage/generate-schema",
      ],
    },
    {
      type: "category",
      label: "GitHub Action",
      collapsed: false,
      link: { type: "doc", id: "github/index" },
      items: ["github/trigger", "github/update"],
    },
    "changelog-format",
    "config",
    "semver-check",
    "faq",
    "why",
    "troubleshooting",
    {
      type: "category",
      label: "Extra",
      collapsed: false,
      link: { type: "doc", id: "extra/index" },
      items: ["extra/releasing-binaries", "extra/single-changelog", "extra/yanked-packages"],
    },
  ],
};

module.exports = sidebars;
