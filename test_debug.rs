use faigz_rs::{FastaFormat, FastaIndex, FastaReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fasta_file = "scerevisiae8.fa.gz";

    println!("Loading index...");
    let index = FastaIndex::new(fasta_file, FastaFormat::Fasta)?;

    println!("Number of sequences: {}", index.num_sequences());

    // Print first few sequence names
    for i in 0..std::cmp::min(5, index.num_sequences()) {
        if let Some(name) = index.sequence_name(i) {
            println!("Sequence {}: '{}'", i, name);
            if let Some(length) = index.sequence_length(&name) {
                println!("  Length: {}", length);
            }
            println!("  Has sequence: {}", index.has_sequence(&name));
        }
    }

    // Test with first sequence
    if let Some(seq_name) = index.sequence_name(0) {
        println!("\nTesting with first sequence: '{}'", seq_name);

        let reader = FastaReader::new(&index)?;

        // Try to fetch full sequence
        match reader.fetch_seq_all(&seq_name) {
            Ok(seq) => println!("Full sequence length: {}", seq.len()),
            Err(e) => println!("Error fetching full sequence: {}", e),
        }

        // Try to fetch a small region
        match reader.fetch_seq(&seq_name, 0, 50) {
            Ok(seq) => println!("First 50 bases: {}", seq),
            Err(e) => println!("Error fetching region: {}", e),
        }
    }

    Ok(())
}
