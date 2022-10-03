use crate::paths::AbsolutePath;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct PnpmWorkspaces {
    pub packages: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct PackageJsonWorkspaces {
    pub workspaces: Vec<PathBuf>,
}

pub enum PackageManager {
    #[allow(dead_code)]
    Berry,
    Npm,
    Pnpm,
    #[allow(dead_code)]
    Pnpm6,
    #[allow(dead_code)]
    Yarn,
}

impl PackageManager {
    /// Returns a list of globs for the package workspace.
    /// NOTE: We return a `Vec<PathBuf>` instead of a `GlobSet` because we
    /// may need to iterate through these globs and a `GlobSet` doesn't allow that.
    ///
    /// # Arguments
    ///
    /// * `root_path`:
    ///
    /// returns: Result<Vec<PathBuf, Global>, Error>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn get_workspace_globs(&self, root_path: &AbsolutePath) -> Result<Vec<PathBuf>> {
        match self {
            PackageManager::Pnpm | PackageManager::Pnpm6 => {
                let workspace_yaml = fs::read_to_string(root_path.join("pnpm-workspace.yaml"))?;
                let workspaces: PnpmWorkspaces = serde_yaml::from_str(&workspace_yaml)?;
                if workspaces.packages.is_empty() {
                    Err(anyhow!("pnpm-workspace.yaml: no packages found. Turborepo requires pnpm workspaces and thus packages to be defined in the root pnpm-workspace.yaml"))
                } else {
                    Ok(workspaces.packages)
                }
            }
            PackageManager::Berry | PackageManager::Npm | PackageManager::Yarn => {
                let package_json_text = fs::read_to_string(root_path.join("package.json"))?;
                let package_json: PackageJsonWorkspaces = serde_json::from_str(&package_json_text)?;

                if package_json.workspaces.is_empty() {
                    Err(anyhow!("pnpm-workspace.yaml: no packages found. Turborepo requires pnpm workspaces and thus packages to be defined in the root pnpm-workspace.yaml"))
                } else {
                    Ok(package_json.workspaces)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::env::current_exe;
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn test_get_workspace_globs() {
        let package_manager = PackageManager::Npm;
        let globs = package_manager
            .get_workspace_globs(&Path::new("../examples/basic"))
            .unwrap();

        assert_eq!(
            globs,
            vec![PathBuf::from("apps/*"), PathBuf::from("packages/*")]
        );
    }

    #[test]
    fn test_get_workspace_ignores() {
        let package_manager = PackageManager::Npm;
        let globs = package_manager
            .get_workspace_ignores(&Path::new("../examples/basic"))
            .unwrap();

        assert_eq!(globs.is_match("node_modules/foo"), true);
        assert_eq!(globs.is_match("bar.js"), false);
    }

    #[test]
    fn test_get_workspaces() {
        let package_manager = PackageManager::Npm;
        let home_path = Path::new("../examples/basic");
        let workspaces = package_manager.get_workspaces(&home_path).unwrap();

        // This is not ideal, but we can't compare with an expected set of paths because
        // the paths are absolute and therefore depend on who's running the test.
        for dir_entry in workspaces {
            assert_eq!(dir_entry.file_name().unwrap(), OsStr::new("package.json"))
        }
    }
}
