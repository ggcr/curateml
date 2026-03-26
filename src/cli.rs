use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    Download {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(short, long)]
        user_agent: Option<String>,

        #[structopt(short, long)]
        workers: Option<usize>,
    },
    Extract {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(long, parse(from_os_str))]
        linguist_path: Option<PathBuf>,

        #[structopt(long)]
        max_file_size: Option<u64>,

        #[structopt(short, long)]
        languages: Option<Vec<String>>,
    },
    Dedupe {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(parse(from_os_str))]
        jsonl_dir: Option<PathBuf>,

        #[structopt(parse(from_os_str))]
        exact_dedup_dir: Option<PathBuf>,

        #[structopt(parse(from_os_str))]
        dest_dir: Option<PathBuf>,
    },
}

// CLI Args
#[derive(Debug, StructOpt)]
#[structopt(name = "codecurator")]
pub struct Opt {
    // download, extract
    #[structopt(subcommand)]
    pub cmd: Command,
}
