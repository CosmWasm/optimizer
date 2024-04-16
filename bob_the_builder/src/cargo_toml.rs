use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CargoToml {
    pub package: Option<Package>,
    pub workspace: Option<Workspace>,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    pub members: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub metadata: Option<Metadata>,
}

#[derive(Default, Deserialize, Debug)]
pub struct Metadata {
    pub build_variants: Option<Vec<String>>,
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
