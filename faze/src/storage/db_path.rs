use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the Faze configuration directory
/// Returns ~/.config/faze on Linux/macOS or %APPDATA%/faze on Windows
/// This is for configuration files only, not for data storage
pub fn get_config_dir() -> Result<PathBuf, std::io::Error> {
    let config_dir = if cfg!(target_os = "windows") {
        // Windows: %APPDATA%/faze
        env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("faze")
    } else {
        // Unix: ~/.config/faze
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".config")
            })
            .join("faze")
    };

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

/// Get the Faze data directory for persistent storage (databases, caches, etc.)
/// Returns ~/.local/share/faze on Linux/macOS or %LOCALAPPDATA%/faze on Windows
pub fn get_data_dir() -> Result<PathBuf, std::io::Error> {
    let data_dir = if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%/faze
        env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                env::var("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."))
            })
            .join("faze")
    } else {
        // Unix: ~/.local/share/faze
        env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".local")
                    .join("share")
            })
            .join("faze")
    };

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

/// Detect the project directory by looking for common project markers
/// Returns the project root or current directory if no marker found
pub fn detect_project_root() -> PathBuf {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut dir = current_dir.as_path();
    loop {
        let markers = [
            ".git",
            "Cargo.toml",
            "package.json",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "pyproject.toml",
            "composer.json",
        ];

        for marker in &markers {
            if dir.join(marker).exists() {
                return dir.to_path_buf();
            }
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    current_dir
}

/// Generate a safe database name from a project path
/// Converts the path into a safe filename by replacing separators with underscores
fn project_path_to_db_name(project_path: &Path) -> String {
    let path_str = project_path.to_string_lossy().to_string();

    let safe_name = path_str
        .replace(['/', '\\', ':', ' '], "_")
        .trim_matches('_')
        .to_string()
        .to_ascii_lowercase();

    if safe_name.len() > 100 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path_str.hash(&mut hasher);
        format!("project_{:x}", hasher.finish())
    } else if safe_name.is_empty() {
        "default".to_string()
    } else {
        safe_name
    }
}

/// Get the database path for the current project
/// Returns a path in ~/.local/share/faze/<project_name>.db on Unix
/// or %LOCALAPPDATA%/faze/<project_name>.db on Windows
pub fn get_project_db_path() -> Result<PathBuf, std::io::Error> {
    let data_dir = get_data_dir()?;
    let project_root = detect_project_root();
    let db_name = project_path_to_db_name(&project_root);

    Ok(data_dir.join(format!("{}.db", db_name)))
}

/// Get the default database path
pub fn get_default_db_path() -> Result<PathBuf, std::io::Error> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join("default.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_path_to_db_name() {
        assert_eq!(
            project_path_to_db_name(Path::new("/home/user/projects/myapp")),
            "home_user_projects_myapp"
        );
        let windows_result = project_path_to_db_name(Path::new("C:\\Users\\user\\projects\\myapp"));
        assert!(windows_result.contains("users_user_projects_myapp"));
        assert_eq!(
            project_path_to_db_name(Path::new("/home/user/my projects/app")),
            "home_user_my_projects_app"
        );
    }

    #[test]
    fn test_project_path_to_db_name_long() {
        let long_path = format!("/home/user/{}", "a".repeat(200));
        let db_name = project_path_to_db_name(Path::new(&long_path));
        assert!(db_name.starts_with("project_"));
        assert!(db_name.len() < 30);
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir().unwrap();
        assert!(config_dir.to_string_lossy().contains("faze"));
    }

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir().unwrap();
        assert!(data_dir.to_string_lossy().contains("faze"));
        #[cfg(not(target_os = "windows"))]
        assert!(
            data_dir.to_string_lossy().contains(".local/share")
                || data_dir.to_string_lossy().contains("XDG_DATA_HOME")
        );
    }

    #[test]
    fn test_detect_project_root() {
        let root = detect_project_root();
        assert!(root.join("Cargo.toml").exists() || root == env::current_dir().unwrap());
    }
}
