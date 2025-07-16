# faigz-rs

[![CI](https://github.com/waveygang/faigz-rs/workflows/CI/badge.svg)](https://github.com/waveygang/faigz-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70.0%2B-blue.svg)](https://www.rust-lang.org)

A Rust wrapper for the faigz reentrant FASTA/FASTQ index library.

This library provides thread-safe, reentrant access to FASTA and FASTQ files using a shared index structure that can be safely accessed from multiple threads. It's built on top of the faigz C library, which provides a fully reentrant faidx implementation.

## Features

- **Thread-safe**: Multiple threads can safely access the same FASTA file concurrently
- **Reentrant**: The index structure is designed for multi-threaded applications
- **Memory efficient**: Index metadata is shared across all readers
- **Format support**: Both FASTA and FASTQ formats are supported
- **Compression support**: Works with bgzip-compressed files
- **Zero-copy**: Efficient memory usage with minimal copying
- **Command-line tool**: Includes a `faigz` binary for extracting sequences like `samtools faidx` and `bedtools getfasta`

## Prerequisites

This library requires **htslib** to be installed on your system. The library will **not work** without it.

### System Requirements

- **htslib**: The HTSlib library for high-throughput sequencing data (required)
- **C compiler**: GCC or Clang for building the C wrapper
- **Rust**: Version 1.70 or later
- **pkg-config**: For finding system libraries

### Installing htslib

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install libhts-dev pkg-config build-essential
```

#### macOS
```bash
brew install htslib
```

#### CentOS/RHEL/Fedora
```bash
# CentOS/RHEL
sudo yum install htslib-devel pkgconfig gcc
# or Fedora
sudo dnf install htslib-devel pkgconfig gcc
```

#### From Source
```bash
git clone https://github.com/samtools/htslib.git
cd htslib
./configure
make
sudo make install
sudo ldconfig  # Linux only
```

### Verification

After installation, verify htslib is available:
```bash
pkg-config --libs htslib
# Should output: -lhts
```

## Installation

### From Git Repository

Add this to your `Cargo.toml`:

```toml
[dependencies]
faigz-rs = { git = "https://github.com/waveygang/faigz-rs" }
```

Or for a specific branch/tag:

```toml
[dependencies]
faigz-rs = { git = "https://github.com/waveygang/faigz-rs", branch = "main" }
```

### Building from Source

1. **Clone the repository with submodules:**
   ```bash
   git clone --recursive https://github.com/waveygang/faigz-rs
   cd faigz-rs
   ```

   Or if you already cloned without `--recursive`:
   ```bash
   git clone https://github.com/waveygang/faigz-rs
   cd faigz-rs
   git submodule update --init --recursive
   ```

2. **Install dependencies:**
   ```bash
   # On Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install build-essential libhts-dev pkg-config libclang-dev
   
   # On other systems, ensure you have:
   # - C compiler (gcc/clang)
   # - HTSlib development headers
   # - pkg-config
   # - libclang (for bindgen)
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

4. **Build the command-line tool:**
   ```bash
   cargo build --release --bin faigz
   ```

5. **Run tests:**
   ```bash
   cargo test
   ```

### Using in Your Project

Once you've added faigz-rs to your dependencies, you can use it in your Rust project:

```rust
use faigz_rs::{FastaIndex, FastaReader, FastaFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = FastaIndex::new("genome.fa", FastaFormat::Fasta)?;
    let reader = FastaReader::new(&index)?;
    let sequence = reader.fetch_seq("chr1", 1000, 1100)?;
    println!("Sequence: {}", sequence);
    Ok(())
}
```

## Command-Line Tool

The `faigz` command-line tool provides a high-performance alternative to `samtools faidx` and `bedtools getfasta`.

### Installation

After building the project, you can install the tool globally:

```bash
cargo install --path . --bin faigz
```

### Usage

```bash
# Create a test FASTA file
faigz create-test-file --output test.fa

# Show sequence information (like samtools faidx -i)
faigz info test.fa

# Extract sequences using 0-based half-open coordinates (bedtools style)
faigz extract test.fa chrX:0-1          # First character
faigz extract test.fa chr1:10-20         # 10 characters from position 10
faigz extract test.fa chr1 chr2          # Entire sequences

# Extract using 1-based coordinates (samtools style)
faigz extract test.fa chr1:11-20 --one-based

# Compare with samtools faidx
faigz compare test.fa chr1:10-20

# Test multithreaded performance
faigz thread-test test.fa --threads 8 --operations 1000
```

### Coordinate Systems

The tool supports both coordinate systems:

- **Default (0-based half-open)**: Compatible with `bedtools getfasta`
  - `chrX:0-1` extracts the first character
  - `chrX:0-5` extracts the first 5 characters
  - Start is inclusive, end is exclusive

- **1-based (--one-based flag)**: Compatible with `samtools faidx`
  - `chrX:1-1` extracts the first character
  - `chrX:1-5` extracts the first 5 characters
  - Both start and end are inclusive

### Performance

The `faigz` tool is highly optimized for multithreaded access:

```bash
# Test with 8 threads extracting 1000 sequences each
faigz thread-test genome.fa --threads 8 --operations 1000

# Typical output:
# Testing with 8 threads, 1000 operations per thread...
# Total: 8000/8000 successful extractions
# Time: 8.2ms
# Rate: 975609.76 extractions/second
```

## Library Usage

### Basic Usage

```rust
use faigz_rs::{FastaIndex, FastaReader, FastaFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the index metadata once
    let index = FastaIndex::new("genome.fa", FastaFormat::Fasta)?;
    
    // Create a reader
    let reader = FastaReader::new(&index)?;
    
    // Fetch sequence data
    let sequence = reader.fetch_seq("chr1", 1000, 1100)?;
    println!("Sequence: {}", sequence);
    
    // Fetch entire sequence
    let full_sequence = reader.fetch_seq_all("chr1")?;
    println!("Full sequence length: {}", full_sequence.len());
    
    // Parse and fetch region
    let region_sequence = reader.fetch_region("chr1:1000-1100")?;
    println!("Region sequence: {}", region_sequence);
    
    Ok(())
}
```

### Multi-threaded Usage

```rust
use faigz_rs::{FastaIndex, FastaReader, FastaFormat};
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the index once
    let index = Arc::new(FastaIndex::new("genome.fa", FastaFormat::Fasta)?);
    
    let mut handles = vec![];
    
    // Create multiple threads that share the same index
    for thread_id in 0..4 {
        let index_clone = Arc::clone(&index);
        
        let handle = thread::spawn(move || {
            // Each thread gets its own reader
            let reader = FastaReader::new(&index_clone).unwrap();
            
            // Access different sequences from each thread
            let seq_name = format!("chr{}", thread_id + 1);
            match reader.fetch_seq_all(&seq_name) {
                Ok(sequence) => {
                    println!("Thread {}: Fetched {} ({} bp)", 
                           thread_id, seq_name, sequence.len());
                }
                Err(e) => {
                    eprintln!("Thread {}: Error: {}", thread_id, e);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}
```

### Working with FASTQ Files

```rust
use faigz_rs::{FastaIndex, FastaReader, FastaFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load FASTQ file
    let index = FastaIndex::new("reads.fq", FastaFormat::Fastq)?;
    let reader = FastaReader::new(&index)?;
    
    // Fetch sequence
    let sequence = reader.fetch_seq("read1", 0, 50)?;
    println!("Sequence: {}", sequence);
    
    // Fetch quality scores (FASTQ only)
    let quality = reader.fetch_qual("read1", 0, 50)?;
    println!("Quality: {}", quality);
    
    Ok(())
}
```

## API Reference

### `FastaIndex`

The main index structure that holds metadata about the FASTA/FASTQ file.

#### Methods

- `new(path: &str, format: FastaFormat) -> FastaResult<Self>`: Create a new index
- `num_sequences(&self) -> usize`: Get number of sequences
- `sequence_name(&self, index: usize) -> Option<String>`: Get sequence name by index
- `sequence_length(&self, name: &str) -> Option<i64>`: Get sequence length
- `has_sequence(&self, name: &str) -> bool`: Check if sequence exists
- `sequence_names(&self) -> Vec<String>`: Get all sequence names

### `FastaReader`

Thread-safe reader for accessing sequences.

#### Methods

- `new(index: &FastaIndex) -> FastaResult<Self>`: Create a new reader
- `fetch_seq(&self, seqname: &str, start: i64, end: i64) -> FastaResult<String>`: Fetch subsequence
- `fetch_seq_all(&self, seqname: &str) -> FastaResult<String>`: Fetch entire sequence
- `fetch_qual(&self, seqname: &str, start: i64, end: i64) -> FastaResult<String>`: Fetch quality scores (FASTQ only)
- `fetch_region(&self, region: &str) -> FastaResult<String>`: Parse region string and fetch

### `FastaFormat`

Enum for specifying file format:
- `FastaFormat::Fasta`: FASTA format
- `FastaFormat::Fastq`: FASTQ format

### Error Handling

The library uses the `FastaError` enum for error handling:

- `InvalidPath`: Invalid file path
- `IndexLoadError`: Failed to load index
- `ReaderCreationError`: Failed to create reader
- `SequenceNotFound`: Sequence not found
- `InvalidRegion`: Invalid region string
- `QualityNotAvailable`: Quality data not available (FASTA format)

## Examples

### Command-Line Examples

```bash
# Create a test file and extract sequences
cargo run --bin faigz create-test-file --output test.fa
cargo run --bin faigz info test.fa
cargo run --bin faigz extract test.fa chrX:0-1 chr1:10-20

# Compare with samtools faidx
cargo run --bin faigz compare test.fa chr1:10-20

# Performance test
cargo run --bin faigz thread-test test.fa --threads 8 --operations 1000
```

### Library Examples

See the [Library Usage](#library-usage) section above for comprehensive examples.

## Testing

Run the test suite:
```bash
cargo test
```

**Note**: htslib is required for all tests to pass. If htslib is not properly installed, the tests will fail with index loading errors.

## Building

To build the library:
```bash
cargo build --release
```

To build the command-line tool:
```bash
cargo build --release --bin faigz
```

To install the command-line tool globally:
```bash
cargo install --path . --bin faigz
```

## Performance

The library is designed for high-performance multi-threaded access. The shared index structure means that:

1. **Memory efficiency**: Index data is loaded once and shared across all threads
2. **Thread safety**: Multiple readers can access the same file concurrently
3. **Scalability**: Performance scales with the number of threads for read-heavy workloads

## Development

### Building from Source

See the [Building from Source](#building-from-source) section above.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in release mode
cargo test --release
```

### Code Quality

This project uses several tools to ensure code quality:

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Continuous Integration

The project uses GitHub Actions for CI/CD with the following checks:

- **Format check**: Ensures code is properly formatted
- **Clippy**: Catches common mistakes and suggests improvements
- **Tests**: Runs the full test suite on Linux
- **Build**: Verifies all targets build successfully
- **MSRV**: Tests minimum supported Rust version (1.70.0)
- **Security audit**: Checks for known security vulnerabilities
- **Dependency check**: Monitors for outdated dependencies

**Note**: This library requires htslib and will not function without it. The old stub implementation has been removed.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Before submitting a pull request:

1. Run the full test suite: `cargo test`
2. Check formatting: `cargo fmt --all -- --check`
3. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings -A clippy::uninlined_format_args`
4. Build examples: `cargo build --examples`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built on top of the [faigz](https://github.com/waveygang/faigz) library
- Uses [HTSlib](https://github.com/samtools/htslib) for FASTA/FASTQ processing
- Inspired by the need for thread-safe FASTA access in bioinformatics applications