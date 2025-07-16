use clap::{Parser, Subcommand};
use faigz_rs::{FastaIndex, FastaReader, FastaFormat};
use std::fs;

#[derive(Parser)]
#[command(name = "faigz")]
#[command(about = "A high-performance tool for extracting sequences from FASTA files, compatible with samtools faidx and bedtools getfasta")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a test FASTA file for demonstration
    CreateTestFile {
        /// Output file path
        #[arg(short, long, default_value = "test.fa")]
        output: String,
    },
    /// Show information about sequences in a FASTA file (like samtools faidx -i)
    Info {
        /// FASTA file path
        fasta: String,
    },
    /// Extract sequences from FASTA file (like samtools faidx and bedtools getfasta)
    Extract {
        /// FASTA file path
        fasta: String,
        /// Regions to extract (format: chr:start-end or chr for whole sequence)
        /// Uses 0-based half-open coordinates like bedtools (start inclusive, end exclusive)
        regions: Vec<String>,
        /// Use 1-based coordinates like samtools faidx instead of 0-based
        #[arg(short, long)]
        one_based: bool,
    },
    /// Test multithreaded access
    ThreadTest {
        /// FASTA file path
        fasta: String,
        /// Number of threads to use
        #[arg(short, long, default_value = "4")]
        threads: usize,
        /// Number of operations per thread
        #[arg(short, long, default_value = "100")]
        operations: usize,
    },
    /// Compare with samtools faidx output
    Compare {
        /// FASTA file path
        fasta: String,
        /// Region to compare
        region: String,
        /// Use 1-based coordinates like samtools faidx
        #[arg(short, long)]
        one_based: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateTestFile { output } => {
            create_test_file(&output)?;
            println!("Created test FASTA file: {}", output);
        }
        Commands::Info { fasta } => {
            show_info(&fasta)?;
        }
        Commands::Extract { fasta, regions, one_based } => {
            extract_sequences(&fasta, &regions, one_based)?;
        }
        Commands::ThreadTest { fasta, threads, operations } => {
            thread_test(&fasta, threads, operations)?;
        }
        Commands::Compare { fasta, region, one_based } => {
            compare_with_samtools(&fasta, &region, one_based)?;
        }
    }

    Ok(())
}

fn create_test_file(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = r#">chr1
ATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCG
ATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCGATCG
>chr2
GCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTA
GCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTA
>chr3
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
>chr4
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT
>chrX
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC
"#;

    fs::write(output, content)?;
    Ok(())
}

fn show_info(fasta: &str) -> Result<(), Box<dyn std::error::Error>> {
    let index = FastaIndex::new(fasta, FastaFormat::Fasta)?;
    
    println!("FASTA file: {}", fasta);
    println!("Number of sequences: {}", index.num_sequences());
    println!();
    
    for i in 0..index.num_sequences() {
        if let Some(name) = index.sequence_name(i) {
            if let Some(length) = index.sequence_length(&name) {
                println!("{}\t{}", name, length);
            }
        }
    }
    
    Ok(())
}

fn extract_sequences(fasta: &str, regions: &[String], one_based: bool) -> Result<(), Box<dyn std::error::Error>> {
    let index = FastaIndex::new(fasta, FastaFormat::Fasta)?;
    let reader = FastaReader::new(&index)?;
    
    for region in regions {
        let result = if region.contains(':') {
            // Parse region like chr1:100-200
            let parts: Vec<&str> = region.split(':').collect();
            if parts.len() != 2 {
                eprintln!("Invalid region format: {}", region);
                continue;
            }
            
            let chr = parts[0];
            let range = parts[1];
            
            if range.contains('-') {
                let range_parts: Vec<&str> = range.split('-').collect();
                if range_parts.len() != 2 {
                    eprintln!("Invalid range format: {}", range);
                    continue;
                }
                
                let start: i64 = range_parts[0].parse().map_err(|e| {
                    format!("Invalid start position '{}': {}", range_parts[0], e)
                })?;
                let end: i64 = range_parts[1].parse().map_err(|e| {
                    format!("Invalid end position '{}': {}", range_parts[1], e)
                })?;
                
                // Convert coordinates based on system
                let (actual_start, actual_end) = if one_based {
                    // samtools faidx uses 1-based inclusive coordinates
                    // Convert to 0-based half-open
                    (start - 1, end)
                } else {
                    // bedtools uses 0-based half-open coordinates (start inclusive, end exclusive)
                    (start, end)
                };
                
                reader.fetch_seq(chr, actual_start, actual_end)
            } else {
                // Single position
                let pos: i64 = range.parse().map_err(|e| {
                    format!("Invalid position '{}': {}", range, e)
                })?;
                
                let actual_pos = if one_based { pos - 1 } else { pos };
                reader.fetch_seq(chr, actual_pos, actual_pos + 1)
            }
        } else {
            // Whole sequence
            reader.fetch_seq_all(region)
        };
        
        match result {
            Ok(sequence) => {
                println!(">{}", region);
                // Print sequence in 80-character lines like standard FASTA
                for line in sequence.as_bytes().chunks(80) {
                    println!("{}", String::from_utf8_lossy(line));
                }
            }
            Err(e) => {
                eprintln!("Error extracting {}: {}", region, e);
            }
        }
    }
    
    Ok(())
}

fn thread_test(fasta: &str, num_threads: usize, operations: usize) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;
    
    let index = Arc::new(FastaIndex::new(fasta, FastaFormat::Fasta)?);
    let sequences = index.sequence_names();
    
    if sequences.is_empty() {
        return Err("No sequences found in FASTA file".into());
    }
    
    println!("Testing with {} threads, {} operations per thread...", num_threads, operations);
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let index_clone = Arc::clone(&index);
        let sequences_clone = sequences.clone();
        
        let handle = thread::spawn(move || {
            let reader = FastaReader::new(&index_clone).unwrap();
            let mut success_count = 0;
            
            for i in 0..operations {
                let seq_name = &sequences_clone[i % sequences_clone.len()];
                let seq_len = index_clone.sequence_length(seq_name).unwrap_or(0);
                
                if seq_len > 10 {
                    // Extract a small region
                    let start = (i as i64) % (seq_len - 10);
                    let end = start + 10;
                    
                    match reader.fetch_seq(seq_name, start, end) {
                        Ok(seq) => {
                            if seq.len() == 10 {
                                success_count += 1;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            
            (thread_id, success_count)
        });
        
        handles.push(handle);
    }
    
    let mut total_success = 0;
    for handle in handles {
        let (thread_id, success_count) = handle.join().unwrap();
        println!("Thread {}: {}/{} successful extractions", thread_id, success_count, operations);
        total_success += success_count;
    }
    
    let duration = start.elapsed();
    println!("\nTotal: {}/{} successful extractions", total_success, num_threads * operations);
    println!("Time: {:?}", duration);
    println!("Rate: {:.2} extractions/second", total_success as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn compare_with_samtools(fasta: &str, region: &str, one_based: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // Extract using faigz-rs
    let index = FastaIndex::new(fasta, FastaFormat::Fasta)?;
    let reader = FastaReader::new(&index)?;
    
    let faigz_result = if region.contains(':') {
        let parts: Vec<&str> = region.split(':').collect();
        let chr = parts[0];
        let range = parts[1];
        let range_parts: Vec<&str> = range.split('-').collect();
        let start: i64 = range_parts[0].parse()?;
        let end: i64 = range_parts[1].parse()?;
        
        let (actual_start, actual_end) = if one_based {
            (start - 1, end)
        } else {
            (start, end)
        };
        
        reader.fetch_seq(chr, actual_start, actual_end)?
    } else {
        reader.fetch_seq_all(region)?
    };
    
    println!("=== faigz-rs result ===");
    println!(">{}", region);
    for line in faigz_result.as_bytes().chunks(80) {
        println!("{}", String::from_utf8_lossy(line));
    }
    
    // Try to compare with samtools faidx if available
    let samtools_region = if one_based {
        region.to_string()
    } else {
        // Convert 0-based to 1-based for samtools
        if region.contains(':') {
            let parts: Vec<&str> = region.split(':').collect();
            let chr = parts[0];
            let range = parts[1];
            let range_parts: Vec<&str> = range.split('-').collect();
            let start: i64 = range_parts[0].parse()?;
            let end: i64 = range_parts[1].parse()?;
            format!("{}:{}-{}", chr, start + 1, end)
        } else {
            region.to_string()
        }
    };
    
    match Command::new("samtools")
        .args(["faidx", fasta, &samtools_region])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("\n=== samtools faidx result ===");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                
                // Compare sequences (skip header line)
                let samtools_seq = String::from_utf8_lossy(&output.stdout);
                let samtools_lines: Vec<&str> = samtools_seq.lines().collect();
                let samtools_sequence = if samtools_lines.len() > 1 {
                    samtools_lines[1..].join("")
                } else {
                    String::new()
                };
                
                if faigz_result == samtools_sequence {
                    println!("✅ Sequences match!");
                } else {
                    println!("❌ Sequences differ!");
                    println!("faigz-rs length: {}", faigz_result.len());
                    println!("samtools length: {}", samtools_sequence.len());
                    if faigz_result.len() < 200 && samtools_sequence.len() < 200 {
                        println!("faigz-rs: {}", faigz_result);
                        println!("samtools: {}", samtools_sequence);
                    }
                }
            } else {
                eprintln!("samtools faidx failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => {
            println!("\n(samtools not available for comparison)");
        }
    }
    
    Ok(())
}