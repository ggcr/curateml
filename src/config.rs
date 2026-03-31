use std::path::{Path, PathBuf};

use crate::cli;

fn bundled_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn invocation_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn source_name(source: &PathBuf) -> Option<String> {
    source
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned())
}

fn derived_source_dir_from(root: &Path, base_dir: &str, source: &PathBuf) -> PathBuf {
    match source_name(source) {
        Some(name) => root.join(base_dir).join(name),
        None => root.join(base_dir),
    }
}

fn derived_source_dir(base_dir: &str, source: &PathBuf) -> PathBuf {
    derived_source_dir_from(&invocation_root(), base_dir, source)
}

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub source: PathBuf,
    pub zip_dir: PathBuf,
    pub user_agent: String,
    pub workers: usize,
}

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub source: PathBuf,
    pub zip_dir: PathBuf,
    pub jsonl_dir: PathBuf,
    pub linguist_path: PathBuf,
    pub max_file_size: u64,
    pub languages: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct DedupeConfig {
    pub source: PathBuf,
    pub jsonl_dir: PathBuf,
    pub exact_dedup_dir: PathBuf,
    pub dest_dir: PathBuf,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            source: bundled_root().join("config/example.jsonl"),
            zip_dir: invocation_root().join("zip"),
            user_agent: "CodeCurator".to_string(),
            workers: 16,
        }
    }
}

impl DownloadConfig {
    pub fn from_cli(opts_cmd: &cli::Command) -> DownloadConfig {
        let mut config = DownloadConfig::default();
        if let cli::Command::Download {
            source,
            user_agent,
            workers,
        } = opts_cmd
        {
            config.source = source.to_owned();
            config.zip_dir = derived_source_dir("zip", source);
            if let Some(u) = user_agent {
                config.user_agent = u.to_owned();
            }
            if let Some(w) = workers {
                config.workers = w.to_owned();
            }
        }
        config
    }
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            source: bundled_root().join("config/repos.jsonl"),
            zip_dir: invocation_root().join("zip"),
            jsonl_dir: invocation_root().join("jsonl"),
            linguist_path: bundled_root().join("vendor/languages.yml"),
            max_file_size: 2u64.pow(17), // 128KB
            languages: None,             // None, Empty, will grab all files
        }
    }
}

impl ExtractionConfig {
    pub fn from_cli(opts_cmd: &cli::Command) -> ExtractionConfig {
        let mut config = ExtractionConfig::default();
        if let cli::Command::Extract {
            source,
            linguist_path,
            max_file_size,
            languages,
        } = opts_cmd
        {
            config.source = source.to_owned();
            config.zip_dir = derived_source_dir("zip", source);
            config.jsonl_dir = derived_source_dir("jsonl", source);
            if let Some(l) = linguist_path {
                config.linguist_path = l.to_owned();
            }
            if let Some(m) = max_file_size {
                config.max_file_size = m.to_owned();
            }
            config.languages = languages.to_owned();
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadConfig, ExtractionConfig, bundled_root, derived_source_dir_from};
    use crate::cli::Command;
    use std::path::{Path, PathBuf};

    #[test]
    fn download_uses_source_stem_for_zip_dir() {
        let cmd = Command::Download {
            source: PathBuf::from("sources/tt09.jsonl"),
            user_agent: None,
            workers: None,
        };
        let cwd = std::env::current_dir().unwrap();

        let config = DownloadConfig::from_cli(&cmd);

        assert_eq!(
            config.zip_dir,
            derived_source_dir_from(&cwd, "zip", &PathBuf::from("sources/tt09.jsonl"))
        );
    }

    #[test]
    fn extract_uses_source_stem_for_zip_and_jsonl_dirs() {
        let cmd = Command::Extract {
            source: PathBuf::from("sources/tt09.jsonl"),
            linguist_path: None,
            max_file_size: None,
            languages: None,
        };
        let cwd = std::env::current_dir().unwrap();

        let config = ExtractionConfig::from_cli(&cmd);

        assert_eq!(
            config.zip_dir,
            derived_source_dir_from(&cwd, "zip", &PathBuf::from("sources/tt09.jsonl"))
        );
        assert_eq!(
            config.jsonl_dir,
            derived_source_dir_from(&cwd, "jsonl", &PathBuf::from("sources/tt09.jsonl"))
        );
        assert_eq!(
            config.linguist_path,
            bundled_root().join("vendor/languages.yml")
        );
    }

    #[test]
    fn derived_source_dir_uses_invocation_root() {
        let root = Path::new("/tmp/invocation-root");
        let source = PathBuf::from("nested/tt09.jsonl");

        assert_eq!(
            derived_source_dir_from(root, "zip", &source),
            root.join("zip/tt09")
        );
    }
}

impl Default for DedupeConfig {
    fn default() -> Self {
        Self {
            source: PathBuf::from("./config/example.jsonl"),
            jsonl_dir: PathBuf::from("./jsonl"),
            exact_dedup_dir: PathBuf::from("./exact"),
            dest_dir: PathBuf::from("./dedup"),
        }
    }
}

impl DedupeConfig {
    pub fn from_cli(opts_cmd: &cli::Command) -> DedupeConfig {
        let mut config = DedupeConfig::default();
        if let cli::Command::Dedupe {
            source,
            jsonl_dir,
            exact_dedup_dir,
            dest_dir,
        } = opts_cmd
        {
            config.source = source.to_owned();
            if let Some(j) = jsonl_dir {
                config.jsonl_dir = j.to_owned();
            }
            if let Some(d) = exact_dedup_dir {
                config.dest_dir = d.to_owned();
            }
            if let Some(d) = dest_dir {
                config.dest_dir = d.to_owned();
            }
        }
        config
    }
}
