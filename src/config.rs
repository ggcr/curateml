use std::path::PathBuf;

use crate::cli;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source_name(source: &PathBuf) -> Option<String> {
    source
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned())
}

fn derived_source_dir(base_dir: &str, source: &PathBuf) -> PathBuf {
    match source_name(source) {
        Some(name) => project_root().join(base_dir).join(name),
        None => project_root().join(base_dir),
    }
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
            source: project_root().join("config/example.jsonl"),
            zip_dir: project_root().join("zip"),
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
            source: project_root().join("config/repos.jsonl"),
            zip_dir: project_root().join("zip"),
            jsonl_dir: project_root().join("jsonl"),
            linguist_path: project_root().join("vendor/languages.yml"),
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
    use super::{DownloadConfig, ExtractionConfig, project_root};
    use crate::cli::Command;
    use std::path::PathBuf;

    #[test]
    fn download_uses_source_stem_for_zip_dir() {
        let cmd = Command::Download {
            source: PathBuf::from("sources/tt09.jsonl"),
            user_agent: None,
            workers: None,
        };

        let config = DownloadConfig::from_cli(&cmd);

        assert_eq!(config.zip_dir, project_root().join("zip/tt09"));
    }

    #[test]
    fn extract_uses_source_stem_for_zip_and_jsonl_dirs() {
        let cmd = Command::Extract {
            source: PathBuf::from("sources/tt09.jsonl"),
            linguist_path: None,
            max_file_size: None,
            languages: None,
        };

        let config = ExtractionConfig::from_cli(&cmd);

        assert_eq!(config.zip_dir, project_root().join("zip/tt09"));
        assert_eq!(config.jsonl_dir, project_root().join("jsonl/tt09"));
        assert_eq!(
            config.linguist_path,
            project_root().join("vendor/languages.yml")
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
