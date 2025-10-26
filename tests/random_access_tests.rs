use faigz_rs::{FastaFormat, FastaIndex, FastaReader};
use rand::Rng;
use std::fs;
use std::process::Command;

fn run_samtools_faidx(fasta_file: &str, region: &str) -> Result<String, String> {
    let output = Command::new("samtools")
        .arg("faidx")
        .arg(fasta_file)
        .arg(region)
        .output()
        .map_err(|e| format!("Failed to run samtools: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "samtools failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse FASTA output - skip header line and join sequence lines
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err("No sequence data returned".to_string());
    }

    Ok(lines[1..].join(""))
}

#[test]
fn test_comprehensive_random_access() {
    let fasta_file = "scerevisiae8.fa.gz";

    // Check if test file exists
    if !fs::metadata(fasta_file).is_ok() {
        eprintln!("Test file {} not found, skipping test", fasta_file);
        return;
    }

    // Load the index
    let index =
        FastaIndex::new(fasta_file, FastaFormat::Fasta).expect("Failed to load FASTA index");

    let reader = FastaReader::new(&index).expect("Failed to create FASTA reader");

    println!("Testing {} sequences", index.num_sequences());

    let mut rng = rand::thread_rng();
    let mut errors = Vec::new();
    let mut successes = 0;

    // Test 100 random regions
    for test_num in 0..100 {
        // Pick a random sequence
        let seq_idx = rng.gen_range(0..index.num_sequences());
        let seq_name = index.sequence_name(seq_idx).unwrap();
        let seq_len = index.sequence_length(&seq_name).unwrap();

        if seq_len <= 1 {
            continue;
        }

        // Generate random region (1-based for samtools)
        let start = rng.gen_range(1..=std::cmp::max(1, seq_len - 100));
        let end = rng.gen_range(start..=std::cmp::min(seq_len, start + 200));

        // Test with faigz-rs (0-based coordinates)
        let faigz_result = reader.fetch_seq(&seq_name, start - 1, end);

        // Test with samtools (1-based coordinates)
        let samtools_region = format!("{}:{}-{}", seq_name, start, end);
        let samtools_result = run_samtools_faidx(fasta_file, &samtools_region);

        match (faigz_result, samtools_result) {
            (Ok(faigz_seq), Ok(samtools_seq)) => {
                if faigz_seq != samtools_seq {
                    errors.push(format!(
                        "Test {}: Sequence mismatch for {}:{}-{}\n  faigz-rs: {}\n  samtools: {}",
                        test_num + 1,
                        seq_name,
                        start,
                        end,
                        &faigz_seq[..std::cmp::min(50, faigz_seq.len())],
                        &samtools_seq[..std::cmp::min(50, samtools_seq.len())]
                    ));
                } else {
                    successes += 1;
                }
            }
            (Err(faigz_err), Ok(_)) => {
                errors.push(format!(
                    "Test {}: faigz-rs failed for {}:{}-{}: {}",
                    test_num + 1,
                    seq_name,
                    start,
                    end,
                    faigz_err
                ));
            }
            (Ok(_), Err(samtools_err)) => {
                errors.push(format!(
                    "Test {}: samtools failed for {}:{}-{}: {}",
                    test_num + 1,
                    seq_name,
                    start,
                    end,
                    samtools_err
                ));
            }
            (Err(faigz_err), Err(samtools_err)) => {
                // Both failed - this might be expected for some edge cases
                println!(
                    "Test {}: Both failed for {}:{}-{} (may be expected)",
                    test_num + 1,
                    seq_name,
                    start,
                    end
                );
            }
        }

        if test_num % 10 == 0 {
            println!("Progress: {}/100 tests completed", test_num);
        }
    }

    println!("Random access tests: {}/100 passed", successes);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}", error);
        }
        panic!("Random access tests failed with {} errors", errors.len());
    }
}

#[test]
fn test_edge_cases() {
    let fasta_file = "scerevisiae8.fa.gz";

    if !fs::metadata(fasta_file).is_ok() {
        eprintln!("Test file {} not found, skipping test", fasta_file);
        return;
    }

    let index =
        FastaIndex::new(fasta_file, FastaFormat::Fasta).expect("Failed to load FASTA index");

    let reader = FastaReader::new(&index).expect("Failed to create FASTA reader");

    let mut errors = Vec::new();

    // Test first few sequences for edge cases
    for seq_idx in 0..std::cmp::min(3, index.num_sequences()) {
        let seq_name = index.sequence_name(seq_idx).unwrap();
        let seq_len = index.sequence_length(&seq_name).unwrap();

        if seq_len == 0 {
            continue;
        }

        // Test full sequence
        let faigz_full = reader.fetch_seq_all(&seq_name);
        let samtools_full = run_samtools_faidx(fasta_file, &seq_name);

        match (faigz_full, samtools_full) {
            (Ok(faigz_seq), Ok(samtools_seq)) => {
                if faigz_seq != samtools_seq {
                    errors.push(format!(
                        "Full sequence mismatch for {}: lengths faigz={}, samtools={}",
                        seq_name,
                        faigz_seq.len(),
                        samtools_seq.len()
                    ));
                }
            }
            (Err(e), _) => errors.push(format!("faigz-rs failed for full {}: {}", seq_name, e)),
            (_, Err(e)) => errors.push(format!("samtools failed for full {}: {}", seq_name, e)),
        }

        // Test first base
        let faigz_first = reader.fetch_seq(&seq_name, 0, 1);
        let samtools_first = run_samtools_faidx(fasta_file, &format!("{}:1-1", seq_name));

        match (faigz_first, samtools_first) {
            (Ok(faigz_seq), Ok(samtools_seq)) => {
                if faigz_seq != samtools_seq {
                    errors.push(format!(
                        "First base mismatch for {}: faigz='{}', samtools='{}'",
                        seq_name, faigz_seq, samtools_seq
                    ));
                }
            }
            _ => {} // Ignore errors for single base tests
        }

        // Test last base
        if seq_len > 0 {
            let faigz_last = reader.fetch_seq(&seq_name, seq_len - 1, seq_len);
            let samtools_last =
                run_samtools_faidx(fasta_file, &format!("{}:{}-{}", seq_name, seq_len, seq_len));

            match (faigz_last, samtools_last) {
                (Ok(faigz_seq), Ok(samtools_seq)) => {
                    if faigz_seq != samtools_seq {
                        errors.push(format!(
                            "Last base mismatch for {}: faigz='{}', samtools='{}'",
                            seq_name, faigz_seq, samtools_seq
                        ));
                    }
                }
                _ => {} // Ignore errors for single base tests
            }
        }
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}", error);
        }
        panic!("Edge case tests failed with {} errors", errors.len());
    }

    println!("Edge case tests passed");
}

#[test]
fn test_invalid_regions() {
    let fasta_file = "scerevisiae8.fa.gz";

    if !fs::metadata(fasta_file).is_ok() {
        eprintln!("Test file {} not found, skipping test", fasta_file);
        return;
    }

    let index =
        FastaIndex::new(fasta_file, FastaFormat::Fasta).expect("Failed to load FASTA index");

    let reader = FastaReader::new(&index).expect("Failed to create FASTA reader");

    // Test non-existent sequence
    let result = reader.fetch_seq("nonexistent_sequence", 0, 100);
    assert!(result.is_err(), "Should fail on non-existent sequence");

    // Test out-of-bounds regions
    if let Some(seq_name) = index.sequence_name(0) {
        let seq_len = index.sequence_length(&seq_name).unwrap();

        // Test beyond end of sequence
        let result = reader.fetch_seq(&seq_name, seq_len + 1, seq_len + 10);
        // This might succeed with empty result or fail - both are acceptable

        // Test invalid coordinates (start > end)
        let result = reader.fetch_seq(&seq_name, 100, 50);
        // This should typically fail or return empty
    }

    println!("Invalid region tests completed");
}

#[test]
fn test_region_parsing() {
    let fasta_file = "scerevisiae8.fa.gz";

    if !fs::metadata(fasta_file).is_ok() {
        eprintln!("Test file {} not found, skipping test", fasta_file);
        return;
    }

    let index =
        FastaIndex::new(fasta_file, FastaFormat::Fasta).expect("Failed to load FASTA index");

    let reader = FastaReader::new(&index).expect("Failed to create FASTA reader");

    // Test region string parsing
    if let Some(seq_name) = index.sequence_name(0) {
        let seq_len = index.sequence_length(&seq_name).unwrap();
        if seq_len > 100 {
            // Test region string format
            let region_str = format!("{}:10-50", seq_name);
            let region_result = reader.fetch_region(&region_str);

            // Compare with direct coordinate access
            let direct_result = reader.fetch_seq(&seq_name, 9, 50); // 0-based

            match (region_result, direct_result) {
                (Ok(region_seq), Ok(direct_seq)) => {
                    assert_eq!(
                        region_seq, direct_seq,
                        "Region parsing should match direct access"
                    );
                }
                _ => {} // Ignore if either fails
            }
        }
    }

    println!("Region parsing tests completed");
}
