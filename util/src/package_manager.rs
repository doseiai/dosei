use std::path::Path;
use std::{fmt, fs};

pub fn _resolve_package_manager(folder_path: &Path) -> Result<PackageManager, &'static str> {
  let folder_path = match fs::canonicalize(folder_path) {
    Ok(path) => path,
    Err(_) => return Err("Invalid folder path"),
  };

  if folder_path.join("poetry.lock").exists() {
    return Ok(PackageManager::Poetry);
  }

  if folder_path.join("requirements.txt").exists() {
    return Ok(PackageManager::Pip);
  }
  Err("No supported package manager found")
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
  fn test_resolve_package_manager_with_poetry() {
    let temp_dir = Builder::new().prefix("example").tempdir().unwrap();
    File::create(temp_dir.path().join("poetry.lock")).unwrap();

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
