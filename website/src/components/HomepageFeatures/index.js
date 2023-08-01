import React from "react";
import clsx from "clsx";
import styles from "./styles.module.css";

const FeatureList = [
  {
    title: "Changelog generation 💡",
    description: (
      <>
        Release-plz updates your changelogs with{" "}
        <a href="https://github.com/orhun/git-cliff">git-cliff</a> using{" "}
        <a href="https://keepachangelog.com/en/1.0.0/">Keep a changelog</a>{" "}
        format by default.
      </>
    ),
  },
  {
    title: "GitHub/Gitea releases 🔎",
    description: <>Release-plz creates GitHub and Gitea tag and releases.</>,
  },
  {
    title: "Publish to crates.io ⚙",
    description: (
      <>
        Release-plz publishes your Rust crates to{" "}
        <a href="https://crates.io/">crates.io</a> by respecting the right
        release order.
      </>
    ),
  },
  {
    title: "Version bump ⚙",
    description: (
      <>
        Release-plz bumps the versions of your crates, updating `Cargo.toml` and
        `Cargo.lock` files. The versions are updated according to{" "}
        <a href="https://semver.org/">Semantic Versioning</a>, based on{" "}
        <a href="https://www.conventionalcommits.org/">Conventional Commits</a>{" "}
        and API breaking changes detected by{" "}
        <a href="https://github.com/obi1kenobi/cargo-semver-checks">
          cargo-semver-checks
        </a>
        .
      </>
    ),
  },
];

function Feature({ title, description }) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
        <video loop controls autoPlay muted style={{ maxWidth: "100%" }}>
          <source src="/release-plz.mp4" type="video/mp4" />
        </video>
      </div>
    </section>
  );
}
