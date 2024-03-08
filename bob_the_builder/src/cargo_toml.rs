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
    use serde::Deserialize;

    type BuildName = String;
    type Feature = String;

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

    #[derive(Deserialize, Debug)]
    pub struct Optimizer {
        builds: Option<Vec<Build>>,
    }

    /// A build entry that specifies the build of a contract with optional features.
    #[derive(Deserialize, Debug, Default)]
    pub struct Build {
        /// Name appended to the build output file name.
        pub name: BuildName,
        /// Features to be enabled for this build.
        pub features: Option<Vec<Feature>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ParsedPackage {
        pub name: String,
        pub builds: Vec<Build>,
    }

    /// Get all the builds and wasm name from the `Cargo.toml` file at the given path.
    pub fn parse_toml(file: &str) -> Result<ParsedPackage, toml::de::Error> {
        let PackageCargoToml { package } = toml::from_str(&file).unwrap();

        // TODO: do in pre-build step
        // let wasm_name = package.name.replace("-", "_");

        let builds = package
            .metadata
            .and_then(|metadata| metadata.optimizer)
            .and_then(|optimizer| optimizer.builds)
            .unwrap_or_default();

        Ok(ParsedPackage {
            name: package.name,
            builds,
        })
    }
}
