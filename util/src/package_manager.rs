use std::path::Path;
use std::{fmt, fs};

pub fn _resolve_docker(folder_path: &Path) -> bool {
  let folder_path = match fs::canonicalize(folder_path) {
    Ok(path) => path,
    Err(_) => return false,
  };
  folder_path.join("Dockerfile").exists()
}

pub fn _resolve_package_manager(folder_path: &Path) -> Result<PackageManager, &'static str> {
  let folder_path = match fs::canonicalize(folder_path) {
    Ok(path) => path,
    Err(_) => return Err("Invalid folder path"),
  };

  if folder_path.join("poetry.lock").exists() || folder_path.join("poetry.toml").exists() {
    return Ok(PackageManager::Poetry);
  }

  if folder_path.join("requirements.txt").exists() {
    return Ok(PackageManager::Pip);
  }
  Err("No supported package manager found")
}

pub fn _resolve_python_version(folder_path: &Path) -> Result<String, String> {
  match _resolve_package_manager(folder_path) {
    Ok(PackageManager::Poetry) => {
      let file_path = folder_path.join("pyproject.toml");
      let contents =
        fs::read_to_string(file_path).map_err(|_| "Failed to read pyproject.toml".to_string())?;
      let data: toml::Value = contents
        .parse()
        .map_err(|_| "Failed to parse TOML".to_string())?;

      let version = data
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(|deps| deps.get("python"))
        .map(|version| version.to_string())
        .ok_or_else(|| "Python version not found in pyproject.toml".to_string())?;

      Ok(version)
    }
    Ok(PackageManager::Pip) => Err("Python version couldn't be auto resolved".to_string()),
    _ => Err("Python version couldn't be auto resolved".to_string()),
  }
}

#[derive(Debug, PartialEq)]
pub enum PackageManager {
  Poetry,
  Pip,
}

impl fmt::Display for PackageManager {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      PackageManager::Poetry => write!(f, "Poetry"),
      PackageManager::Pip => write!(f, "Pip"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::Builder;

  #[test]
  fn test_resolve_package_manager_with_poetry_lock() {
    let temp_dir = Builder::new().prefix("example").tempdir().unwrap();
    File::create(temp_dir.path().join("poetry.lock")).unwrap();

    let result = _resolve_package_manager(temp_dir.path());
    assert_eq!(result, Ok(PackageManager::Poetry));
  }

  #[test]
  fn test_resolve_package_manager_with_poetry_toml() {
    let temp_dir = Builder::new().prefix("example").tempdir().unwrap();
    File::create(temp_dir.path().join("poetry.toml")).unwrap();

    let result = _resolve_package_manager(temp_dir.path());
    assert_eq!(result, Ok(PackageManager::Poetry));
  }

  #[test]
  fn test_resolve_package_manager_with_pip() {
    let temp_dir = Builder::new().prefix("example").tempdir().unwrap();
    File::create(temp_dir.path().join("requirements.txt")).unwrap();

    let result = _resolve_package_manager(temp_dir.path());
    assert_eq!(result, Ok(PackageManager::Pip));
  }

  #[test]
  fn test_resolve_package_manager_with_invalid_path() {
    let result = _resolve_package_manager(Path::new("/invalid/path"));
    assert_eq!(result, Err("Invalid folder path"));
  }

  #[test]
  fn test_resolve_package_manager_no_package_manager() {
    let temp_dir = Builder::new().prefix("example").tempdir().unwrap();
    let result = _resolve_package_manager(temp_dir.path());
    assert_eq!(result, Err("No supported package manager found"));
  }
}
