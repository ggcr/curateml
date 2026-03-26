# CodeCurator

An end-to-end tool for curating GitHub repositories into structured code datasets.

- **Fast parallel processing** - Download and extract with configurable workers
- **Smart filtering** - Only processes programming files using GitHub Linguist
- **GPT-2 tokenization** - Ready-to-use token counts for ML workflows
- **Efficient caching** - Uses ETags to avoid re-downloading unchanged repos

Perfect for curating training data, running code analysis, or creating repository archives.


### Installation

```bash
cargo install --path .
```

### Usage

Create an input file with one GitHub repository per line:

```jsonl
"microsoft/vscode"
"vercel/next.js"
"tensorflow/tensorflow"
"bitcoin/bitcoin"
"rust-lang/rust"
"kubernetes/kubernetes"
"facebook/react"
"docker/compose"
"ansible/ansible"
"elastic/elasticsearch"
```

**Download repositories:**
```bash
codecurator download ./configs/repos.jsonl
```

This creates ZIP files in `./zip/repos/`. Downloads from `main` branch first, falls back to `master` if needed.

**Extract and process:**
```bash
codecurator extract ./configs/repos.jsonl --languages Python Rust Verilog
```

Processes all programming files from `./zip/repos/`, tokenizes content, and outputs structured data to `./jsonl/repos/`.

**Deduplication:**
```bash
codecurator dedupe ./configs/repos.jsonl
```
Hashes the contents of all files and deduplicates them. Stores the final data to `/dedup/` by default.

**Statistics:**
```bash
$ bash stats/count_records.sh ./jsonl/
Total records: 110645

$ bash stats/count_tokens.sh ./dedup/
Total tokens: 346574283
```

![Visitors](https://visitor-badge.laobi.icu/badge?page_id=ggcr.codecurator)
