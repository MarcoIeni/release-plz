use anyhow::Context;

use crate::DepTable;
use std::str;

/// A Cargo manifest
#[derive(Debug, Clone)]
pub struct Manifest {
    /// Manifest contents as TOML data
    pub data: toml_edit::Document,
}

impl Manifest {
    /// Get the specified table from the manifest.
    ///
    /// If there is no table at the specified path, then a non-existent table
    /// error will be returned.
    pub(crate) fn get_table_mut<'a>(
        &'a mut self,
        table_path: &[String],
    ) -> anyhow::Result<&'a mut toml_edit::Item> {
        self.get_table_mut_internal(table_path, false)
    }

    /// Get all sections in the manifest that exist and might contain dependencies.
    /// The returned items are always `Table` or `InlineTable`.
    pub(crate) fn get_sections(&self) -> Vec<(DepTable, toml_edit::Item)> {
        let mut sections = Vec::new();

        for table in DepTable::KINDS {
            let dependency_type = table.kind_table();
            // Dependencies can be in the three standard sections...
            if self
                .data
                .get(dependency_type)
                .map(|t| t.is_table_like())
                .unwrap_or(false)
            {
                sections.push((table.clone(), self.data[dependency_type].clone()))
            }

            // ... and in `target.<target>.(build-/dev-)dependencies`.
            let target_sections = self
                .data
                .as_table()
                .get("target")
                .and_then(toml_edit::Item::as_table_like)
                .into_iter()
                .flat_map(toml_edit::TableLike::iter)
                .filter_map(|(target_name, target_table)| {
                    let dependency_table = target_table.get(dependency_type)?;
                    dependency_table.as_table_like().map(|_| {
                        (
                            table.clone().set_target(target_name),
                            dependency_table.clone(),
                        )
                    })
                });

            sections.extend(target_sections);
        }

        sections
    }

    fn get_table_mut_internal<'a>(
        &'a mut self,
        table_path: &[String],
        insert_if_not_exists: bool,
    ) -> anyhow::Result<&'a mut toml_edit::Item> {
        /// Descend into a manifest until the required table is found.
        fn descend<'a>(
            input: &'a mut toml_edit::Item,
            path: &[String],
            insert_if_not_exists: bool,
        ) -> anyhow::Result<&'a mut toml_edit::Item> {
            if let Some(segment) = path.get(0) {
                let value = if insert_if_not_exists {
                    input[&segment].or_insert(toml_edit::table())
                } else {
                    input
                        .get_mut(segment)
                        .ok_or_else(|| non_existent_table_err(segment))?
                };

                if value.is_table_like() {
                    descend(value, &path[1..], insert_if_not_exists)
                } else {
                    Err(non_existent_table_err(segment))
                }
            } else {
                Ok(input)
            }
        }

        descend(self.data.as_item_mut(), table_path, insert_if_not_exists)
    }
}

impl str::FromStr for Manifest {
    type Err = anyhow::Error;

    /// Read manifest data from string
    fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
        let d: toml_edit::Document = input.parse().context("Manifest not valid TOML")?;

        Ok(Manifest { data: d })
    }
}

impl std::fmt::Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.data.to_string();
        s.fmt(f)
    }
}

fn non_existent_table_err(table: &str) -> anyhow::Error {
    anyhow::anyhow!("The table `{}` could not be found.", table)
}
