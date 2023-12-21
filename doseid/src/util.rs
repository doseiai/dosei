use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn find_framework_init(framework: Framework, folder_path: &Path) -> Result<String, &'static str> {
  let pattern = Regex::new(&format!(r"(\w+)\s*=\s*{}\(", framework.class_name())).unwrap();

  let folder_path = match fs::canonicalize(folder_path) {
    Ok(path) => path,
    Err(_) => return Err("Invalid folder path"),
  };

  for entry in WalkDir::new(&folder_path) {
    let entry = match entry {
      Ok(e) => e,
      Err(_) => continue,
    };

    let path = entry.path();

    if path.is_file() && path.extension().map_or(false, |e| e == "py") {
      let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => continue,
      };

      if let Some(captures) = pattern.captures(&content) {
        // Calculate the relative module path
        let relative_path = match path.strip_prefix(&folder_path) {
          Ok(rp) => rp,
          Err(_) => continue,
        }
        .with_extension("");

        let module_path = relative_path
          .to_str()
          .unwrap()
          .replace(std::path::MAIN_SEPARATOR, ".");

        return Ok(format!("{}:{}", module_path, &captures[1]));
      }
    }
  }
  Err("No framework initialization found.")
}

// TODO: Support Django and Flask
enum Framework {
  Dosei,
  FastAPI,
}

impl Framework {
  fn class_name(&self) -> &str {
    match self {
      Framework::Dosei => "Dosei",
      Framework::FastAPI => "FastAPI",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use std::io::Write;
  use std::path::PathBuf;
  use tempfile::tempdir;

  fn create_file(dir: &Path, file_name: &str, content: &str) {
    let file_path = dir.join(file_name);
    let mut file = File::create(file_path).unwrap();
    writeln!(file, "{}", content).unwrap();
  }

  #[test]
  fn test_no_framework_initialization() {
    let temp_dir = tempdir().unwrap();
    create_file(temp_dir.path(), "test.py", "print('Hello')");
    let result = find_framework_init(Framework::Dosei, temp_dir.path());
    assert!(result.is_err());
  }

  #[test]
  fn test_invalid_directory_path() {
    let invalid_path = PathBuf::from("/invalid/path");
    let result = find_framework_init(Framework::Dosei, &invalid_path);
    assert!(result.is_err());
  }

  #[test]
  fn test_unsupported_file_extension() {
    let temp_dir = tempdir().unwrap();
    create_file(temp_dir.path(), "test.txt", "Dosei = Dosei()");
    let result = find_framework_init(Framework::Dosei, temp_dir.path());
    assert!(result.is_err());
  }

  #[test]
  fn test_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let result = find_framework_init(Framework::Dosei, temp_dir.path());
    assert!(result.is_err());
  }

  #[test]
  fn test_nested_directory_structure() {
    let temp_dir = tempdir().unwrap();
    let nested_dir = temp_dir.path().join("nested");
    fs::create_dir(&nested_dir).unwrap();
    create_file(&nested_dir, "test.py", "Dosei = Dosei()");

    let result = find_framework_init(Framework::Dosei, temp_dir.path());
    assert!(result.is_ok());
  }

  #[test]
  fn test_dosei_framework_specific() {
    let temp_dir = tempdir().unwrap();
    create_file(temp_dir.path(), "test.py", "Dosei = Dosei()");
    let result = find_framework_init(Framework::Dosei, temp_dir.path());
    assert!(result.is_ok());
  }

  #[test]
  fn test_fastapi_framework_specific() {
    let temp_dir = tempdir().unwrap();
    create_file(temp_dir.path(), "test.py", "FastAPI = FastAPI()");
    let result = find_framework_init(Framework::FastAPI, temp_dir.path());
    assert!(result.is_ok());
  }
}
