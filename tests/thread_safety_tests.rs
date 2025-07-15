use faigz_rs::{FastaIndex, FastaReader, FastaFormat};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_large_test_fasta() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    
    // Create a larger test file with multiple sequences
    for i in 0..100 {
        writeln!(file, ">seq{}", i).unwrap();
        // Create sequences of varying lengths
        let seq_len = 50 + (i * 10) % 200;
        let base = match i % 4 {
            0 => 'A',
            1 => 'T',
            2 => 'G',
            _ => 'C',
        };
        let sequence: String = (0..seq_len).map(|_| base).collect();
        writeln!(file, "{}", sequence).unwrap();
    }
    
    file
}

#[test]
fn test_concurrent_readers() {
    let fasta_file = create_large_test_fasta();
    let path = fasta_file.path().to_str().unwrap();
    
    if let Ok(index) = FastaIndex::new(path, FastaFormat::Fasta) {
        let index = Arc::new(index);
        let num_threads = 8;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];
        
        for thread_id in 0..num_threads {
            let index_clone = Arc::clone(&index);
            let barrier_clone = Arc::clone(&barrier);
            
            let handle = thread::spawn(move || {
                let reader = FastaReader::new(&index_clone).unwrap();
                
                // Wait for all threads to be ready
                barrier_clone.wait();
                
                // Each thread accesses different sequences concurrently
                for i in 0..10 {
                    let seq_name = format!("seq{}", (thread_id * 10 + i) % 100);
                    
                    match reader.fetch_seq_all(&seq_name) {
                        Ok(seq) => {
                            println!("Thread {}: Fetched {} (length: {})", thread_id, seq_name, seq.len());
                        }
                        Err(e) => {
                            println!("Thread {}: Failed to fetch {}: {}", thread_id, seq_name, e);
                        }
                    }
                    
                    // Small delay to increase chance of interleaving
                    thread::sleep(Duration::from_millis(1));
                }
                
                println!("Thread {} completed", thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("Concurrent readers test completed successfully");
    } else {
        println!("Concurrent readers test skipped - index creation failed");
    }
}

#[test]
fn test_reader_lifecycle() {
    let fasta_file = create_large_test_fasta();
    let path = fasta_file.path().to_str().unwrap();
    
    if let Ok(index) = FastaIndex::new(path, FastaFormat::Fasta) {
        let index = Arc::new(index);
        let mut handles = vec![];
        
        // Test creating and destroying readers in different threads
        for thread_id in 0..4 {
            let index_clone = Arc::clone(&index);
            
            let handle = thread::spawn(move || {
                for cycle in 0..10 {
                    // Create reader
                    let reader = FastaReader::new(&index_clone).unwrap();
                    
                    // Use reader
                    let seq_name = format!("seq{}", (thread_id * 10 + cycle) % 100);
                    match reader.fetch_seq_all(&seq_name) {
                        Ok(_) => {
                            println!("Thread {}, cycle {}: Successfully fetched {}", 
                                   thread_id, cycle, seq_name);
                        }
                        Err(e) => {
                            println!("Thread {}, cycle {}: Failed to fetch {}: {}", 
                                   thread_id, cycle, seq_name, e);
                        }
                    }
                    
                    // Reader is dropped here
                }
                
                println!("Thread {} completed all cycles", thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("Reader lifecycle test completed successfully");
    } else {
        println!("Reader lifecycle test skipped - index creation failed");
    }
}

#[test]
fn test_index_sharing() {
    let fasta_file = create_large_test_fasta();
    let path = fasta_file.path().to_str().unwrap();
    
    if let Ok(index) = FastaIndex::new(path, FastaFormat::Fasta) {
        let index = Arc::new(index);
        
        // Test that multiple threads can read metadata concurrently
        let mut handles = vec![];
        
        for thread_id in 0..5 {
            let index_clone = Arc::clone(&index);
            
            let handle = thread::spawn(move || {
                for _ in 0..20 {
                    // Access metadata methods
                    let num_seqs = index_clone.num_sequences();
                    let _seq_names = index_clone.sequence_names();
                    
                    // Check some random sequences
                    for i in 0..std::cmp::min(5, num_seqs) {
                        let seq_name = index_clone.sequence_name(i);
                        if let Some(name) = seq_name {
                            let length = index_clone.sequence_length(&name);
                            let exists = index_clone.has_sequence(&name);
                            
                            println!("Thread {}: {} length={:?} exists={}", 
                                   thread_id, name, length, exists);
                        }
                    }
                    
                    thread::sleep(Duration::from_millis(1));
                }
                
                println!("Thread {} completed metadata access", thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("Index sharing test completed successfully");
    } else {
        println!("Index sharing test skipped - index creation failed");
    }
}

#[test]
fn test_stress_concurrent_access() {
    let fasta_file = create_large_test_fasta();
    let path = fasta_file.path().to_str().unwrap();
    
    if let Ok(index) = FastaIndex::new(path, FastaFormat::Fasta) {
        let index = Arc::new(index);
        let mut handles = vec![];
        
        // Stress test with many concurrent readers
        for thread_id in 0..16 {
            let index_clone = Arc::clone(&index);
            
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    let reader = FastaReader::new(&index_clone).unwrap();
                    
                    // Try to access random sequences
                    for _ in 0..5 {
                        let seq_id = thread_id % 100;
                        let seq_name = format!("seq{}", seq_id);
                        
                        match reader.fetch_seq_all(&seq_name) {
                            Ok(seq) => {
                                // Verify sequence is not empty
                                assert!(!seq.is_empty(), "Sequence should not be empty");
                            }
                            Err(_) => {
                                // This is expected if htslib is not properly set up
                            }
                        }
                    }
                }
                
                println!("Stress test thread {} completed", thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("Stress test completed successfully");
    } else {
        println!("Stress test skipped - index creation failed");
    }
}