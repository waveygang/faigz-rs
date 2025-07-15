# faigz-rs

A Rust wrapper for the faigz reentrant FASTA/FASTQ index library.

This library provides thread-safe, reentrant access to FASTA and FASTQ files using a shared index structure that can be safely accessed from multiple threads. It's built on top of the faigz C library, which provides a fully reentrant faidx implementation.

## Features

- **Thread-safe**: Multiple threads can safely access the same FASTA file concurrently
- **Reentrant**: The index structure is designed for multi-threaded applications
- **Memory efficient**: Index metadata is shared across all readers
- **Format support**: Both FASTA and FASTQ formats are supported
- **Compression support**: Works with bgzip-compressed files
- **Zero-copy**: Efficient memory usage with minimal copying

## Prerequisites

Before using this library, you need to have the following installed:

- **htslib**: The HTSlib library for high-throughput sequencing data
- **C compiler**: GCC or Clang for building the C wrapper
- **Rust**: Version 1.70 or later

### Installing htslib

On Ubuntu/Debian:
```bash
sudo apt-get install libhts-dev
```

On macOS:
```bash
brew install htslib
```

On other systems, you can build from source:
```bash
git clone https://github.com/samtools/htslib.git
cd htslib
./configure
make
sudo make install
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
faigz-rs = "0.1.0"
```

## Usage

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

Run the basic usage example:
```bash
cargo run --example basic_usage genome.fa chr1
```

Run the performance demonstration:
```bash
cargo run --example performance_demo genome.fa 8 1000
```

## Testing

Run the test suite:
```bash
cargo test
```

Note: Some tests may be skipped if htslib is not properly installed, as they require actual FASTA file processing.

## Building

To build the library:
```bash
cargo build --release
```

## Performance

The library is designed for high-performance multi-threaded access. The shared index structure means that:

1. **Memory efficiency**: Index data is loaded once and shared across all threads
2. **Thread safety**: Multiple readers can access the same file concurrently
3. **Scalability**: Performance scales with the number of threads for read-heavy workloads

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built on top of the [faigz](https://github.com/waveygang/faigz) library
- Uses [HTSlib](https://github.com/samtools/htslib) for FASTA/FASTQ processing
- Inspired by the need for thread-safe FASTA access in bioinformatics applications