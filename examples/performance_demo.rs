use faigz_rs::{FastaFormat, FastaIndex, FastaReader};
use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <fasta_file> [num_threads] [num_reads_per_thread]",
            args[0]
        );
        eprintln!("Example: {} genome.fa 8 1000", args[0]);
        std::process::exit(1);
    }

    let fasta_path = &args[1];
    let num_threads: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let reads_per_thread: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(100);

    println!(
        "Performance demonstration with {} threads, {} reads per thread",
        num_threads, reads_per_thread
    );

    // Load the index
    let start = Instant::now();
    let index = match FastaIndex::new(fasta_path, FastaFormat::Fasta) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("Error loading index: {}", e);
            std::process::exit(1);
        }
    };
    let load_time = start.elapsed();

    println!("Index loaded in {:?}", load_time);
    println!("Number of sequences: {}", index.num_sequences());

    // Get sequence names for testing
    let sequence_names = index.sequence_names();
    if sequence_names.is_empty() {
        eprintln!("No sequences found in the index");
        std::process::exit(1);
    }

    // Single-threaded baseline
    println!("\n=== Single-threaded baseline ===");
    let start = Instant::now();
    single_threaded_test(&index, &sequence_names, reads_per_thread * num_threads);
    let single_time = start.elapsed();
    println!("Single-threaded time: {:?}", single_time);

    // Multi-threaded test
    println!("\n=== Multi-threaded test ===");
    let start = Instant::now();
    multi_threaded_test(index, &sequence_names, num_threads, reads_per_thread);
    let multi_time = start.elapsed();
    println!("Multi-threaded time: {:?}", multi_time);

    // Calculate speedup
    let speedup = single_time.as_secs_f64() / multi_time.as_secs_f64();
    println!("\nSpeedup: {:.2}x", speedup);
    println!("Efficiency: {:.2}%", (speedup / num_threads as f64) * 100.0);
}

fn single_threaded_test(index: &FastaIndex, sequence_names: &[String], total_reads: usize) {
    let reader = match FastaReader::new(index) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error creating reader: {}", e);
            return;
        }
    };

    let mut successful_reads = 0;
    let mut total_bases = 0;

    for i in 0..total_reads {
        let seq_name = &sequence_names[i % sequence_names.len()];

        match reader.fetch_seq_all(seq_name) {
            Ok(sequence) => {
                successful_reads += 1;
                total_bases += sequence.len();
            }
            Err(_) => {
                // Count failed reads but don't print errors
            }
        }
    }

    println!("Successful reads: {}/{}", successful_reads, total_reads);
    println!("Total bases read: {}", total_bases);
}

fn multi_threaded_test(
    index: FastaIndex,
    sequence_names: &[String],
    num_threads: usize,
    reads_per_thread: usize,
) {
    let index = Arc::new(index);
    let sequence_names: Vec<String> = sequence_names.to_vec();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let index_clone = Arc::clone(&index);
        let sequence_names_clone = sequence_names.clone();

        let handle = thread::spawn(move || {
            let reader = match FastaReader::new(&index_clone) {
                Ok(reader) => reader,
                Err(e) => {
                    eprintln!("Thread {}: Error creating reader: {}", thread_id, e);
                    return (0, 0);
                }
            };

            let mut successful_reads = 0;
            let mut total_bases = 0;

            for i in 0..reads_per_thread {
                let seq_index = (thread_id * reads_per_thread + i) % sequence_names_clone.len();
                let seq_name = &sequence_names_clone[seq_index];

                match reader.fetch_seq_all(seq_name) {
                    Ok(sequence) => {
                        successful_reads += 1;
                        total_bases += sequence.len();
                    }
                    Err(_) => {
                        // Count failed reads but don't print errors
                    }
                }
            }

            (successful_reads, total_bases)
        });

        handles.push(handle);
    }

    // Collect results from all threads
    let mut total_successful = 0;
    let mut total_bases = 0;

    for (thread_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok((successful, bases)) => {
                total_successful += successful;
                total_bases += bases;
                println!(
                    "Thread {}: {}/{} successful reads, {} bases",
                    thread_id, successful, reads_per_thread, bases
                );
            }
            Err(_) => {
                eprintln!("Thread {} panicked", thread_id);
            }
        }
    }

    println!(
        "Total successful reads: {}/{}",
        total_successful,
        num_threads * reads_per_thread
    );
    println!("Total bases read: {}", total_bases);
}

// Additional utility function to demonstrate region-based access
#[allow(dead_code)]
fn region_based_benchmark(index: FastaIndex, sequence_names: &[String], num_threads: usize) {
    println!("\n=== Region-based access benchmark ===");

    let index = Arc::new(index);
    let sequence_names: Vec<String> = sequence_names.to_vec();
    let mut handles = vec![];

    let start = Instant::now();

    for thread_id in 0..num_threads {
        let index_clone = Arc::clone(&index);
        let sequence_names_clone = sequence_names.clone();

        let handle = thread::spawn(move || {
            let reader = match FastaReader::new(&index_clone) {
                Ok(reader) => reader,
                Err(e) => {
                    eprintln!("Thread {}: Error creating reader: {}", thread_id, e);
                    return 0;
                }
            };

            let mut successful_reads = 0;

            // Each thread reads different regions from the same sequences
            for seq_name in &sequence_names_clone {
                if let Some(seq_len) = index_clone.sequence_length(seq_name) {
                    let region_size = std::cmp::min(1000, seq_len / 10);
                    let start_pos =
                        (thread_id as i64 * region_size) % (seq_len - region_size).max(1);
                    let end_pos = start_pos + region_size;

                    if reader.fetch_seq(seq_name, start_pos, end_pos).is_ok() {
                        successful_reads += 1;
                    }
                }
            }

            successful_reads
        });

        handles.push(handle);
    }

    let mut total_successful = 0;
    for handle in handles {
        if let Ok(successful) = handle.join() {
            total_successful += successful;
        }
    }

    let region_time = start.elapsed();
    println!("Region-based access time: {:?}", region_time);
    println!("Total successful region reads: {}", total_successful);
}
