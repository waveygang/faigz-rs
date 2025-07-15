use faigz_rs::{FastaFormat, FastaIndex, FastaReader};
use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <fasta_file> [sequence_name]", args[0]);
        eprintln!("Example: {} genome.fa chr1", args[0]);
        std::process::exit(1);
    }

    let fasta_path = &args[1];
    let sequence_name = args.get(2).map(|s| s.as_str());

    println!("Loading FASTA index from: {}", fasta_path);

    // Load the index
    let index = match FastaIndex::new(fasta_path, FastaFormat::Fasta) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("Error loading index: {}", e);
            std::process::exit(1);
        }
    };

    println!("Index loaded successfully!");
    println!("Number of sequences: {}", index.num_sequences());

    // Print sequence information
    println!("\nSequence information:");
    for i in 0..index.num_sequences() {
        if let Some(name) = index.sequence_name(i) {
            if let Some(length) = index.sequence_length(&name) {
                println!("  {}: {} bp", name, length);
            }
        }
    }

    // Create a reader
    let reader = match FastaReader::new(&index) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error creating reader: {}", e);
            std::process::exit(1);
        }
    };

    // If a sequence name was provided, fetch it
    if let Some(seq_name) = sequence_name {
        println!("\nFetching sequence: {}", seq_name);

        match reader.fetch_seq_all(seq_name) {
            Ok(sequence) => {
                println!("Length: {} bp", sequence.len());

                // Print first 100 characters
                if sequence.len() > 100 {
                    println!("First 100 bp: {}...", &sequence[..100]);
                } else {
                    println!("Sequence: {}", sequence);
                }
            }
            Err(e) => {
                eprintln!("Error fetching sequence: {}", e);
            }
        }

        // Test region fetching
        println!("\nTesting region fetching:");
        match reader.fetch_region(&format!("{}:1-50", seq_name)) {
            Ok(region) => {
                println!("Region {}:1-50: {}", seq_name, region);
            }
            Err(e) => {
                eprintln!("Error fetching region: {}", e);
            }
        }
    }

    // Demonstrate multithreaded access
    println!("\nDemonstrating multithreaded access:");
    demonstrate_multithreaded_access(index);
}

fn demonstrate_multithreaded_access(index: FastaIndex) {
    let index = Arc::new(index);
    let mut handles = vec![];

    // Create multiple threads that share the same index
    for thread_id in 0..4 {
        let index_clone = Arc::clone(&index);

        let handle = thread::spawn(move || {
            let reader = match FastaReader::new(&index_clone) {
                Ok(reader) => reader,
                Err(e) => {
                    eprintln!("Thread {}: Error creating reader: {}", thread_id, e);
                    return;
                }
            };

            // Each thread accesses different sequences
            let seq_names = index_clone.sequence_names();
            if !seq_names.is_empty() {
                let seq_index = thread_id % seq_names.len();
                let seq_name = &seq_names[seq_index];

                match reader.fetch_seq_all(seq_name) {
                    Ok(sequence) => {
                        println!(
                            "Thread {}: Fetched {} ({} bp)",
                            thread_id,
                            seq_name,
                            sequence.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Error fetching {}: {}", thread_id, seq_name, e);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Multithreaded access demonstration completed!");
}
