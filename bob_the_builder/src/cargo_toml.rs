pub mod workspace {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct CargoToml {
        pub workspace: Option<Workspace>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Workspace {
        pub members: Option<Vec<String>>,
    }

    #[derive(Debug, PartialEq)]
    pub enum IsWorkspace {
        Yes {
            members: Vec<String>,
        },
        /// If the members key is not set or empty. This is an error case.
        NoMembers,
        No,
    }

    /// Detects if this is a workspace or not
    pub fn is_workspace(file: &str) -> Result<IsWorkspace, toml::de::Error> {
        let parsed: CargoToml = toml::from_str(file)?;

        if let Some(workspace) = parsed.workspace {
            if let Some(members) = workspace.members {
                if !members.is_empty() {
                    Ok(IsWorkspace::Yes { members })
                } else {
                    Ok(IsWorkspace::NoMembers)
                }
            } else {
                Ok(IsWorkspace::NoMembers)
            }
        } else {
            Ok(IsWorkspace::No)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn is_workspace_works() {
            let is = is_workspace(
                r#"
            title = 'TOML Example'

            [workspace]
            "#,
            )
            .unwrap();
            assert_eq!(is, IsWorkspace::NoMembers);
            let is = is_workspace(
                r#"
            title = 'TOML Example'

            [workspace]
            members = []
            "#,
            )
            .unwrap();
            assert_eq!(is, IsWorkspace::NoMembers);

            let is = is_workspace(
                r#"
            title = 'TOML Example'
            "#,
            )
            .unwrap();
            assert_eq!(is, IsWorkspace::No);

            let is = is_workspace(
                r#"
            title = 'TOML Example'

            [workspace]
            members = ["contracts/*"]
            "#,
            )
            .unwrap();
            assert_eq!(
                is,
                IsWorkspace::Yes {
                    members: vec!["contracts/*".to_string()]
                }
            );
        }
    }
}

pub mod package {
    use std::{collections::BTreeSet, hash::Hash};

    use serde::Deserialize;

    use crate::pkg_build::ParsedPackage;

    pub type BuildName = String;
    pub type Feature = String;

    #[derive(Deserialize, Debug)]
    pub struct PackageCargoToml {
        package: Package,
    }

    #[derive(Deserialize, Debug)]
    pub struct Package {
        name: String,
        metadata: Option<OptimizerMetadata>,
    }

    #[derive(Deserialize, Debug)]
    pub struct OptimizerMetadata {
        optimizer: Option<Optimizer>,
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "kebab-case")]
    pub struct Optimizer {
        /// Indicates if a standard build (a build without explicit features) should be created.
        /// Defaults to true.
        standard_build: Option<bool>,
        /// A collection of named build configurations.
        builds: Option<Vec<Build>>,
    }

    /// A build entry that specifies the build of a contract with optional features.
    #[derive(Deserialize, Debug, Default, PartialEq, Eq, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct Build {
        /// Name appended to the build output file name.
        pub name: BuildName,
        #[serde(flatten)]
        pub settings: BuildSettings,
    }

    #[derive(Clone, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
    #[serde(rename_all = "kebab-case")]
    pub struct BuildSettings {
        /// Features to be enabled for this build.
        pub features: Option<BTreeSet<Feature>>,
        /// Indicates if default features should be enabled for this build.
        /// Default to true.
        pub default_features: Option<bool>,
    }

    /// Get all the builds and wasm name from the `Cargo.toml` file.
    pub fn parse_toml(file: &str) -> Result<ParsedPackage, toml::de::Error> {
        let PackageCargoToml { package } = toml::from_str(file).unwrap();

        let optimizer = package
            .metadata
            .and_then(|metadata| metadata.optimizer)
            .unwrap_or_default();

        Ok(ParsedPackage {
            name: package.name.replace("-", "_"),
            standard_build: optimizer.standard_build.unwrap_or(true),
            builds: optimizer.builds.unwrap_or_default(),
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_default_toml_works() {
            let toml = r#"
            [package]
            name = "my-contract"
            "#;

            let parsed = parse_toml(toml).unwrap();

            assert_eq!(
                parsed,
                ParsedPackage {
                    name: "my_contract".to_string(),
                    standard_build: true,
                    builds: vec![]
                }
            );
        }

        #[test]
        fn parse_toml_works() {
            let toml = r#"
            [package]
            name = "my-contract"

            [package.metadata.optimizer]
            standard-build = false
            builds = [
                { name = "optimized", features = ["opt1", "opt2"], default-features = true },
                { name = "debug", features = ["debug"] },
                { name = "boring", features = [], default-features = false },
            ]
            "#;

            let parsed = parse_toml(toml).unwrap();

            assert_eq!(
                parsed,
                ParsedPackage {
                    name: "my_contract".to_string(),
                    standard_build: false,
                    builds: vec![
                        Build {
                            name: "optimized".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::from([
                                    "opt1".to_string(),
                                    "opt2".to_string()
                                ])),
                                default_features: Some(true)
                            }
                        },
                        Build {
                            name: "debug".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::from(["debug".to_string()])),
                                default_features: None,
                            }
                        },
                        Build {
                            name: "boring".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::default()),
                                default_features: Some(false)
                            }
                        }
                    ]
                }
            );
        }
    }
}
