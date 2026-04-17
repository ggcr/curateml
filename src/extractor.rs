use crate::error::ExtractionError;

use colored::Colorize;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokenizers::Tokenizer;
use uuid::Uuid;
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub text: String,
    pub id: String,
    file_extension: String,
    category: String,
    pub path: String,
    size_in_bytes: u64,
    file_name: String,
    tokens: usize,
}

fn parse_ext(file: &zip::read::ZipFile<'_, BufReader<fs::File>>) -> Option<String> {
    let path = file.enclosed_name()?;
    let ext = path.extension()?.to_str()?;
    Some(format!(".{ext}"))
}

fn get_zip_name(zip_path: &Path) -> Option<&str> {
    let stem = zip_path.file_stem()?.to_str()?; // From OsStr → &str
    let base_name = stem.rsplit_once('_').map(|(left, _)| left).unwrap_or(stem);
    Some(base_name)
}

pub fn write_repo_jsonl(
    dest_dir: &Path,
    file_name: &str,
    r: &Record,
    fc: &i64,
) -> Result<(), ExtractionError> {
    let mut jsonl_name = file_name.to_owned();
    jsonl_name.push_str(".jsonl");
    let jsonl_path = dest_dir.join(jsonl_name);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(*fc == 0)
        .append(*fc != 0)
        .open(&jsonl_path)?;

    let mut writer = BufWriter::new(file);
    jsonl::write(&mut writer, r)?;
    writer.flush()?;
    Ok(())
}

fn extract_path_metadata(
    file: &mut zip::read::ZipFile<'_, BufReader<fs::File>>,
) -> Option<(String, String)> {
    let path: PathBuf = file.enclosed_name()?;
    let filename = path.file_name()?;
    Some((path.display().to_string(), filename.display().to_string()))
}

//
// JSONL format: text, id, path, metadata
//
fn process_valid_file(
    file: &mut zip::read::ZipFile<'_, BufReader<fs::File>>,
    tokenizer: &Tokenizer,
    extension: String,
) -> Result<Record, ExtractionError> {
    // Read file contents
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let text = text.trim().to_string();

    // Secondary fields: id, path
    let id = Uuid::new_v4().to_string();
    let Some((file_path, file_name)): Option<(String, String)> = extract_path_metadata(file) else {
        return Err(ExtractionError::Validation {
            message: format!(
                "Cannot safely extract path and filename from {}.",
                file.name()
            ),
        });
    };

    // Metadata: file_type & tokenize
    let file_type = String::from("programming");
    let Ok(encoding) = tokenizer.encode(text.clone(), false) else {
        return Err(ExtractionError::Tokenizer {
            message: format!("Unable to tokenize {}", file.name()),
        });
    };
    let n_tokens = encoding.len();

    Ok(Record {
        text,
        id,
        category: file_type,
        path: file_path,
        file_name,
        file_extension: extension,
        size_in_bytes: file.size(),
        tokens: n_tokens,
    })
}

fn extract_zip(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
    file_types: &HashMap<String, String>,
    dest_dir: &Path,
    tokenizer: &Tokenizer,
) -> Result<i64, ExtractionError> {
    let mut file_count = 0;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        // If we are in a file + it has extension
        let Some(ext) = parse_ext(&file) else {
            continue;
        };
        if file.is_file() && file.size() <= 2u64.pow(17) && file_types.contains_key(&ext) {
            // Parse file
            let r = match process_valid_file(&mut file, tokenizer, ext) {
                Ok(r) => r,
                Err(_) => {
                    continue;
                }
            };
            // Write to JSONL
            let Ok(_) = write_repo_jsonl(dest_dir, name, &r, &file_count) else {
                continue;
            };
            file_count += 1;
        }
    }
    Ok(file_count)
}

pub fn extract_text(
    jsonl_dir: &PathBuf,
    zip_paths: Vec<PathBuf>,
    file_types: HashMap<String, String>,
    tokenizer: Tokenizer,
) -> Result<(), ExtractionError> {
    let destination_dir = jsonl_dir;
    fs::create_dir_all(&destination_dir)?;

    // Arc types for read-only on async
    let file_types = Arc::new(file_types);
    let tokenizer = Arc::new(tokenizer);
    let dest_dir = Arc::new(destination_dir);

    let total_files: i64 = zip_paths
        .par_iter()
        .map(|zip_path| {
            let ft = Arc::new(&file_types);
            let tok = Arc::new(&tokenizer);
            let ddir = Arc::clone(&dest_dir);

            let f = match fs::File::open(zip_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Unable to open zip {}: {}", zip_path.display(), e);
                    return 0;
                }
            };

            let Some(zip_name) = get_zip_name(zip_path) else {
                return 0;
            };

            let reader = BufReader::new(f);
            let mut zip = zip::ZipArchive::new(reader).unwrap();

            if let Ok(count) = extract_zip(&mut zip, zip_name, &ft, &ddir, &tok) {
                println!("\t{}:  {}", "Extracted".green(), zip_name);
                return count;
            }
            0
        })
        .sum();

    println!("Total files processed = {}", total_files);
    // TODO: Make this error if and only if the original length is not 0
    if total_files <= 0 {
        return Err(ExtractionError::Validation {
            message: String::from("No repos were extracted."),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::extractor::Record;
    use std::path::Path;

    #[test]
    fn path_extension_extracts_last_suffix() {
        let path = Path::new("rtl/foo.test.sv");
        assert_eq!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("sv")
        );
    }

    #[test]
    fn path_extension_ignores_dotfiles_without_a_real_suffix() {
        let path = Path::new(".gitignore");
        assert_eq!(path.extension().and_then(|ext| ext.to_str()), None);
    }

    #[test]
    fn test_record_creation() {
        let record = Record {
            text: "fn main() {}".to_string(),
            id: "test-id".to_string(),
            file_extension: ".rs".to_string(),
            category: "programming".to_string(),
            path: "test.rs".to_string(),
            size_in_bytes: 13,
            file_name: "test.rs".to_string(),
            tokens: 5,
        };

        assert_eq!(record.tokens, 5);
        assert_eq!(record.file_extension, ".rs");
    }
}
