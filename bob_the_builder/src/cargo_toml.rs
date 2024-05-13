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
        let parsed: CargoToml = toml::from_str(&file)?;

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
    pub struct Optimizer {
        #[serde(rename = "default-build")]
        default_build: Option<bool>,
        builds: Option<Vec<Build>>,
    }

    /// A build entry that specifies the build of a contract with optional features.
    #[derive(Deserialize, Debug, Default, PartialEq, Eq)]
    pub struct Build {
        /// Name appended to the build output file name.
        pub name: BuildName,
        #[serde(flatten)]
        pub settings: BuildSettings,
    }

    #[derive(Clone, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
    pub struct BuildSettings {
        /// Features to be enabled for this build.
        pub features: Option<BTreeSet<Feature>>,
    }

    /// Get all the builds and wasm name from the `Cargo.toml` file.
    pub fn parse_toml(file: &str) -> Result<ParsedPackage, toml::de::Error> {
        let PackageCargoToml { package } = toml::from_str(&file).unwrap();

        let optimizer = package
            .metadata
            .and_then(|metadata| metadata.optimizer)
            .unwrap_or_default();

        Ok(ParsedPackage {
            name: package.name.replace("-", "_"),
            default_build: optimizer.default_build.unwrap_or(true),
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
                    default_build: true,
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
            default-build = false
            builds = [
                { name = "optimized", features = ["opt1", "opt2"] },
                { name = "debug", features = ["debug"] },
                { name = "boring", features = [] },
            ]
            "#;

            let parsed = parse_toml(toml).unwrap();

            assert_eq!(
                parsed,
                ParsedPackage {
                    name: "my_contract".to_string(),
                    default_build: false,
                    builds: vec![
                        Build {
                            name: "optimized".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::from([
                                    "opt1".to_string(),
                                    "opt2".to_string()
                                ]))
                            }
                        },
                        Build {
                            name: "debug".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::from(["debug".to_string()]))
                            }
                        },
                        Build {
                            name: "boring".to_string(),
                            settings: BuildSettings {
                                features: Some(BTreeSet::default())
                            }
                        }
                    ]
                }
            );
        }
    }
}
