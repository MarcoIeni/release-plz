import React from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <h1 className="hero__title">RELEASE-PLZ</h1>
        <img
          alt="Release-plz Logo"
          src="img/robot.jpeg"
          style={{ maxHeight: 500, maxWidth: "100%" }}
        />
        <p className="hero__subtitle">
          Release Rust crates from <b>CI</b> with a <b>Release PR</b> 🤖
        </p>
        <div className={styles.buttons}>
          <Link
            style={{ marginRight: 10 }}
            className="button button--secondary button--lg"
            to="/docs"
          >
            Get Started️
          </Link>
          <span className={styles.indexCtasGitHubButtonWrapper} style={{ marginLeft: 10 }}>
            <iframe
              className={styles.indexCtasGitHubButton}
              src="https://ghbtns.com/github-btn.html?user=MarcoIeni&amp;repo=release-plz&amp;type=star&amp;count=true&amp;size=large"
              width={160}
              height={30}
              title="GitHub Stars"
            />
          </span>
        </div>
      </div>
    </header>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} Rust crate`}
      description="Release Rust crates from CI with a Release PR"
    >
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
