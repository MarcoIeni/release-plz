use anyhow::Context;
use secrecy::SecretString;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";
const CRATES_IO_REGISTRY: &str = "crates-io";

/// Find the URL of a registry
pub fn registry_url(manifest_path: &Path, registry: Option<&str>) -> anyhow::Result<Url> {
    fn read_config(
        registries: &mut HashMap<String, Source>,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        // TODO unit test for source replacement
        let content = fs_err::read_to_string(path).context("failed to read cargo config file")?;
        let config = toml::from_str::<CargoConfig>(&content).context("Invalid cargo config")?;
        for (key, value) in config.registries {
            registries.entry(key).or_insert(Source {
                registry: value.index,
                replace_with: None,
            });
        }
        for (key, value) in config.source {
            registries.entry(key).or_insert(value);
        }
        Ok(())
    }
    // registry might be replaced with another source
    // it's looks like a singly linked list
    // put relations in this map.
    let mut registries: HashMap<String, Source> = HashMap::new();
    // ref: https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure
    for work_dir in manifest_path
        .parent()
        .expect("there must be a parent directory")
        .ancestors()
    {
        let work_cargo_dir = work_dir.join(".cargo");
        let config_path = work_cargo_dir.join("config");
        if config_path.is_file() {
            read_config(&mut registries, config_path)?;
        } else {
            let config_path = work_cargo_dir.join("config.toml");
            if config_path.is_file() {
                read_config(&mut registries, config_path)?;
            }
        }
    }

    let default_cargo_home = cargo_home()?;
    let default_config_path = default_cargo_home.join("config");
    if default_config_path.is_file() {
        read_config(&mut registries, default_config_path)?;
    } else {
        let default_config_path = default_cargo_home.join("config.toml");
        if default_config_path.is_file() {
            read_config(&mut registries, default_config_path)?;
        }
    }

    // find head of the relevant linked list
    let mut source = match registry {
        Some(CRATES_IO_INDEX) | None => {
            let mut source = registries.remove(CRATES_IO_REGISTRY).unwrap_or_default();
            source
                .registry
                .get_or_insert_with(|| CRATES_IO_INDEX.to_string());
            source
        }
        Some(r) => registries
            .remove(r)
            .with_context(|| anyhow::anyhow!("The registry '{}' could not be found", r))?,
    };

    // search this linked list and find the tail
    while let Some(replace_with) = &source.replace_with {
        let is_crates_io = replace_with == CRATES_IO_INDEX;
        source = registries
            .remove(replace_with)
            .with_context(|| anyhow::anyhow!("The source '{}' could not be found", replace_with))?;
        if is_crates_io {
            source
                .registry
                .get_or_insert_with(|| CRATES_IO_INDEX.to_string());
        }
    }

    let registry_url = source
        .registry
        .and_then(|x| Url::parse(&x).ok())
        .context("Invalid cargo config")?;

    Ok(registry_url)
}

pub fn registry_token(registry: &Option<String>) -> anyhow::Result<Option<SecretString>> {
    let mut token = registry_token_from_env(registry);
    if token.is_none() {
        token = registry_token_from_credential_file(registry)?;
    }
    Ok(token)
}

/// Read credentials for a specific registry using environment variables.
/// <https://doc.rust-lang.org/cargo/reference/registry-authentication.html#cargotoken>
pub fn registry_token_from_env(registry: &Option<String>) -> Option<SecretString> {
    let token = if let Some(r) = registry {
        let env_var = format!("CARGO_REGISTRIES_{}_TOKEN", r.to_uppercase());
        std::env::var(env_var)
    } else {
        std::env::var("CARGO_REGISTRY_TOKEN")
    };
    token.ok().map(SecretString::new)
}

/// Read credentials for a specific registry using file cargo/credentials.toml.
/// <https://doc.rust-lang.org/cargo/reference/config.html#credentials>
pub fn registry_token_from_credential_file(
    registry: &Option<String>,
) -> anyhow::Result<Option<SecretString>> {
    let mut path = cargo_home()?.join("credentials.toml");
    if !path.exists() {
        path = cargo_home()?.join("credentials");
    }
    if !path.exists() {
        return Ok(None);
    }
    let content = fs_err::read_to_string(path).context("failed to read cargo credentials file")?;
    let credentials =
        toml::from_str::<CargoCredentials>(&content).context("Invalid cargo credentials file")?;
    let token = if let Some(r) = registry {
        credentials.registries.get(r)
    } else {
        credentials.registry.as_ref()
    }
    .and_then(|r| r.token.clone())
    .map(SecretString::new);
    Ok(token)
}

#[derive(Debug, Deserialize)]
struct CargoConfig {
    #[serde(default)]
    registries: HashMap<String, Registry>,
    #[serde(default)]
    source: HashMap<String, Source>,
}

#[derive(Default, Debug, Deserialize)]
struct Source {
    #[serde(rename = "replace-with")]
    replace_with: Option<String>,
    registry: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Registry {
    index: Option<String>,
}

#[derive(Debug, Deserialize, Default, PartialEq)]
struct CargoCredentials {
    #[serde(default)]
    registry: Option<RegistryToken>,
    #[serde(default)]
    registries: HashMap<String, RegistryToken>,
}

#[derive(Debug, Deserialize, Default, PartialEq)]
struct RegistryToken {
    token: Option<String>,
}

fn cargo_home() -> anyhow::Result<PathBuf> {
    let default_cargo_home = dirs::home_dir()
        .map(|x| x.join(".cargo"))
        .context("Failed to read home directory")?;
    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or(default_cargo_home);
    Ok(cargo_home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_credentials_both() {
        let sample = r#"
            [registry]
            token = "aaaa"   # Access token for crates.io

            [registries.my]
            token = "bbb"   # Access token for the named registry
        "#;
        let creds = toml::from_str::<CargoCredentials>(sample).unwrap();
        assert_eq!(
            creds.registry.and_then(|r| r.token),
            Some("aaaa".to_string())
        );
        assert_eq!(
            creds.registries.get("my").and_then(|r| r.token.clone()),
            Some("bbb".to_string())
        );
        assert_eq!(
            creds.registries.get("foo").and_then(|r| r.token.clone()),
            None
        );
    }

    #[test]
    fn test_parse_cargo_credentials_cratesio_only() {
        let sample = r#"
            [registry]
            token = "aaaa"   # Access token for crates.io
        "#;
        let creds = toml::from_str::<CargoCredentials>(sample).unwrap();
        assert_eq!(
            creds.registry.and_then(|r| r.token),
            Some("aaaa".to_string())
        );
        assert_eq!(
            creds.registries.get("my").and_then(|r| r.token.clone()),
            None
        );
        assert_eq!(
            creds.registries.get("foo").and_then(|r| r.token.clone()),
            None
        );
    }

    #[test]
    fn test_parse_cargo_credentials_empty() {
        let sample = "";
        let creds = toml::from_str::<CargoCredentials>(sample).unwrap();
        assert_eq!(creds.registry.and_then(|r| r.token), None);
        assert_eq!(
            creds.registries.get("my").and_then(|r| r.token.clone()),
            None
        );
        assert_eq!(
            creds.registries.get("foo").and_then(|r| r.token.clone()),
            None
        );
    }
}
