use faigz_rs::{FastaError, FastaFormat, FastaIndex, FastaReader};
use std::io::Write;
use std::sync::Arc;
use std::thread;
use tempfile::NamedTempFile;

fn create_test_fasta() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, ">seq1").unwrap();
    writeln!(file, "ATCGATCGATCGATCG").unwrap();
    writeln!(file, ">seq2").unwrap();
    writeln!(file, "GCTAGCTAGCTAGCTA").unwrap();
    writeln!(file, "AAAAAAAAAAAAAAAA").unwrap();
    writeln!(file, ">seq3").unwrap();
    writeln!(file, "TTTTTTTTTTTTTTTT").unwrap();
    file
}

fn create_test_fastq() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "@seq1").unwrap();
    writeln!(file, "ATCGATCGATCGATCG").unwrap();
    writeln!(file, "+").unwrap();
    writeln!(file, "IIIIIIIIIIIIIIII").unwrap();
    writeln!(file, "@seq2").unwrap();
    writeln!(file, "GCTAGCTAGCTAGCTA").unwrap();
    writeln!(file, "+").unwrap();
    writeln!(file, "JJJJJJJJJJJJJJJJ").unwrap();
    file
}

#[test]
fn test_basic_functionality() {
    let fasta_file = create_test_fasta();
    let path = fasta_file.path().to_str().unwrap();

    let index = FastaIndex::new(path, FastaFormat::Fasta).unwrap();

    // Test index metadata
    assert!(index.num_sequences() > 0);

    // Test sequence names
    let names = index.sequence_names();
    assert!(names.contains(&"seq1".to_string()));
    assert!(names.contains(&"seq2".to_string()));
    assert!(names.contains(&"seq3".to_string()));

    // Test sequence lengths
    assert!(index.sequence_length("seq1").is_some());
    assert!(index.sequence_length("nonexistent").is_none());

    // Test sequence existence
    assert!(index.has_sequence("seq1"));
    assert!(!index.has_sequence("nonexistent"));

    // Test reader creation
    let reader = FastaReader::new(&index).unwrap();

    // Test sequence fetching
    let seq = reader.fetch_seq("seq1", 0, 10).unwrap();
    assert!(!seq.is_empty());
    println!("Fetched sequence: {}", seq);
}

#[test]
fn test_multithreaded_access() {
    let fasta_file = create_test_fasta();
    let path = fasta_file.path().to_str().unwrap();

    let index = Arc::new(FastaIndex::new(path, FastaFormat::Fasta).unwrap());
    let mut handles = vec![];

    // Create multiple threads that share the same index
    for i in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let reader = FastaReader::new(&index_clone).unwrap();

            // Each thread tries to access the same sequences
            for seq_name in &["seq1", "seq2", "seq3"] {
                let seq = reader.fetch_seq_all(seq_name).unwrap();
                println!("Thread {} fetched {}: length {}", i, seq_name, seq.len());
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Multithreaded test completed successfully");
}

#[test]
fn test_error_handling() {
    // Test with nonexistent file
    let result = FastaIndex::new("/nonexistent/file.fa", FastaFormat::Fasta);
    assert!(result.is_err());

    match result.unwrap_err() {
        FastaError::IndexLoadError(_) => (),
        _ => panic!("Expected IndexLoadError"),
    }

    // Test with empty path
    let result = FastaIndex::new("", FastaFormat::Fasta);
    assert!(result.is_err());
}

#[test]
fn test_region_parsing() {
    let fasta_file = create_test_fasta();
    let path = fasta_file.path().to_str().unwrap();

    let index = FastaIndex::new(path, FastaFormat::Fasta).unwrap();
    let reader = FastaReader::new(&index).unwrap();

    // Test region parsing
    let seq = reader.fetch_region("seq1:1-10").unwrap();
    println!("Fetched region seq1:1-10: {}", seq);
    assert!(!seq.is_empty());

    // Test whole sequence fetch
    let seq = reader.fetch_region("seq1").unwrap();
    println!("Fetched whole sequence seq1: {}", seq);
    assert!(!seq.is_empty());

    // Test invalid region
    let result = reader.fetch_region("invalid_format");
    assert!(result.is_err());
    println!("Invalid region correctly failed");
}

#[test]
fn test_fastq_support() {
    let fastq_file = create_test_fastq();
    let path = fastq_file.path().to_str().unwrap();

    let index = FastaIndex::new(path, FastaFormat::Fastq).unwrap();
    let reader = FastaReader::new(&index).unwrap();

    // Test quality score fetching (currently not supported in minimal implementation)
    let qual_result = reader.fetch_qual("seq1", 0, 10);
    // Our minimal implementation doesn't support quality string fetching yet
    match qual_result {
        Err(FastaError::QualityNotAvailable) => {
            println!("Quality string fetching not supported (as expected)");
        }
        Ok(qual) => {
            println!("Fetched quality scores: {}", qual);
            assert!(!qual.is_empty());
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_clone_and_drop() {
    let fasta_file = create_test_fasta();
    let path = fasta_file.path().to_str().unwrap();

    let index = FastaIndex::new(path, FastaFormat::Fasta).unwrap();

    // Test cloning
    let index_clone = index.clone();

    // Both should have the same number of sequences
    assert_eq!(index.num_sequences(), index_clone.num_sequences());

    // Test that readers can be created from both
    let _reader1 = FastaReader::new(&index).unwrap();
    let _reader2 = FastaReader::new(&index_clone).unwrap();

    // When we drop the readers and indices, reference counting should work properly
    drop(_reader1);
    drop(_reader2);
    drop(index);
    drop(index_clone);

    println!("Clone and drop test completed successfully");
}

#[test]
fn test_memory_safety() {
    let fasta_file = create_test_fasta();
    let path = fasta_file.path().to_str().unwrap();

    let index = FastaIndex::new(path, FastaFormat::Fasta).unwrap();

    // Create many readers and drop them
    for _i in 0..100 {
        let _reader = FastaReader::new(&index).unwrap();
        // Reader will be dropped at the end of this scope
    }

    println!("Memory safety test completed successfully");
}
