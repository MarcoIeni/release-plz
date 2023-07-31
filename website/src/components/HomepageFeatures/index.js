import React from "react";
import clsx from "clsx";
import styles from "./styles.module.css";

const FeatureList = [
  {
    title: "Mnemonic 💡",
    description: (
      <>
        The most useful VSCode Commands organized by mnemonic keys, like{" "}
        <code>f</code> for <i>file</i> and <code>r</code> for <i>refactor</i>.
        Plus, if you use Vim motions, you'll feel at home.
      </>
    ),
  },
  {
    title: "Discoverable 🔎",
    description: (
      <>
        Discover new VSCode Commands you wish you knew before, and execute them
        efficiently from your keyboard.
      </>
    ),
  },
  {
    title: "Customizable ⚙",
    description: (
      <>
        No need to learn yet another configuration syntax: edit the Release-plz
        menu and key bindings with plain JavaScript.
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
