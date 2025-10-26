#!/usr/bin/env python3
"""
Comprehensive test script for faigz-rs random access functionality.
Compares the behavior of faigz-rs against samtools faidx as reference.
"""

import subprocess
import sys
import random
import json
from pathlib import Path
import tempfile
import os

def run_command(cmd, capture_output=True, text=True):
    """Run a command and return result."""
    try:
        result = subprocess.run(cmd, capture_output=capture_output, text=text, shell=True)
        return result.returncode, result.stdout, result.stderr
    except Exception as e:
        return -1, "", str(e)

def get_fasta_info(fasta_file):
    """Get sequence info from FASTA file using samtools."""
    cmd = f"samtools faidx {fasta_file} && cut -f1,2 {fasta_file}.fai"
    rc, stdout, stderr = run_command(cmd)
    if rc != 0:
        print(f"Error getting FASTA info: {stderr}")
        return []
    
    sequences = []
    for line in stdout.strip().split('\n'):
        if line:
            name, length = line.split('\t')
            sequences.append((name, int(length)))
    return sequences

def samtools_fetch(fasta_file, seqname, start=None, end=None):
    """Fetch sequence using samtools faidx."""
    if start is None and end is None:
        region = seqname
    else:
        region = f"{seqname}:{start}-{end}"
    
    cmd = f"samtools faidx {fasta_file} {region}"
    rc, stdout, stderr = run_command(cmd)
    if rc != 0:
        return None, f"samtools error: {stderr}"
    
    # Parse FASTA output - skip header line
    lines = stdout.strip().split('\n')
    if len(lines) < 2:
        return None, "No sequence data returned"
    
    sequence = ''.join(lines[1:])  # Join all sequence lines
    return sequence, None

def faigz_fetch(fasta_file, seqname, start=None, end=None):
    """Fetch sequence using faigz-rs demo binary."""
    if start is None and end is None:
        region = seqname
    else:
        # Convert to 0-based coordinates for faigz-rs
        region = f"{seqname}:{start-1}-{end}"
    
    cmd = f"./target/debug/faigz extract {fasta_file} '{region}'"
    rc, stdout, stderr = run_command(cmd)
    if rc != 0:
        return None, f"faigz-rs error: {stderr}"
    
    # faigz extract outputs FASTA format - skip header line and join sequence lines
    lines = stdout.strip().split('\n')
    if len(lines) < 2:
        return None, "No sequence data returned"
    
    sequence = ''.join(lines[1:])  # Join all sequence lines
    return sequence, None

def test_random_regions(fasta_file, sequences, num_tests=100):
    """Test random regions and compare faigz-rs vs samtools."""
    errors = []
    successes = 0
    
    print(f"Testing {num_tests} random regions...")
    
    for i in range(num_tests):
        # Pick random sequence
        seqname, seq_len = random.choice(sequences)
        
        # Generate random region
        if seq_len <= 1:
            continue
            
        start = random.randint(1, max(1, seq_len - 100))
        end = random.randint(start, min(seq_len, start + 200))
        
        # Fetch with both tools
        sam_seq, sam_err = samtools_fetch(fasta_file, seqname, start, end)
        faigz_seq, faigz_err = faigz_fetch(fasta_file, seqname, start, end)
        
        if sam_err:
            errors.append(f"Test {i+1}: samtools error for {seqname}:{start}-{end}: {sam_err}")
            continue
            
        if faigz_err:
            errors.append(f"Test {i+1}: faigz-rs error for {seqname}:{start}-{end}: {faigz_err}")
            continue
            
        if sam_seq != faigz_seq:
            errors.append(f"Test {i+1}: Sequence mismatch for {seqname}:{start}-{end}")
            errors.append(f"  samtools: {sam_seq[:50]}{'...' if len(sam_seq) > 50 else ''}")
            errors.append(f"  faigz-rs: {faigz_seq[:50]}{'...' if len(faigz_seq) > 50 else ''}")
            continue
            
        successes += 1
        if i % 10 == 0:
            print(f"  Progress: {i}/{num_tests} tests completed")
    
    print(f"Random region tests: {successes}/{num_tests} passed")
    return errors

def test_edge_cases(fasta_file, sequences):
    """Test edge cases and boundary conditions."""
    errors = []
    
    print("Testing edge cases...")
    
    for seqname, seq_len in sequences:
        # Test full sequence
        sam_seq, sam_err = samtools_fetch(fasta_file, seqname)
        faigz_seq, faigz_err = faigz_fetch(fasta_file, seqname)
        
        if sam_err or faigz_err:
            errors.append(f"Edge case: Full sequence {seqname} failed")
            if sam_err:
                errors.append(f"  samtools: {sam_err}")
            if faigz_err:
                errors.append(f"  faigz-rs: {faigz_err}")
            continue
            
        if sam_seq != faigz_seq:
            errors.append(f"Edge case: Full sequence {seqname} mismatch")
            errors.append(f"  Length diff: samtools={len(sam_seq)}, faigz-rs={len(faigz_seq)}")
            continue
        
        # Test first base
        sam_seq, sam_err = samtools_fetch(fasta_file, seqname, 1, 1)
        faigz_seq, faigz_err = faigz_fetch(fasta_file, seqname, 1, 1)
        
        if not sam_err and not faigz_err and sam_seq != faigz_seq:
            errors.append(f"Edge case: First base {seqname}:1-1 mismatch")
            errors.append(f"  samtools: '{sam_seq}', faigz-rs: '{faigz_seq}'")
        
        # Test last base
        if seq_len > 0:
            sam_seq, sam_err = samtools_fetch(fasta_file, seqname, seq_len, seq_len)
            faigz_seq, faigz_err = faigz_fetch(fasta_file, seqname, seq_len, seq_len)
            
            if not sam_err and not faigz_err and sam_seq != faigz_seq:
                errors.append(f"Edge case: Last base {seqname}:{seq_len}-{seq_len} mismatch")
                errors.append(f"  samtools: '{sam_seq}', faigz-rs: '{faigz_seq}'")
    
    print(f"Edge case tests completed")
    return errors

def test_invalid_regions(fasta_file, sequences):
    """Test invalid regions and error handling."""
    errors = []
    
    print("Testing invalid regions...")
    
    # Test non-existent sequence
    sam_seq, sam_err = samtools_fetch(fasta_file, "nonexistent_seq")
    faigz_seq, faigz_err = faigz_fetch(fasta_file, "nonexistent_seq")
    
    if not sam_err and not faigz_err:
        errors.append("Invalid region: Both tools should fail on non-existent sequence")
    
    # Test out-of-bounds regions
    for seqname, seq_len in sequences[:3]:  # Test first 3 sequences
        if seq_len > 0:
            # Test beyond end
            sam_seq, sam_err = samtools_fetch(fasta_file, seqname, seq_len + 1, seq_len + 10)
            faigz_seq, faigz_err = faigz_fetch(fasta_file, seqname, seq_len + 1, seq_len + 10)
            
            # Both should handle this gracefully (may return empty or error)
            if not sam_err and not faigz_err and sam_seq != faigz_seq:
                errors.append(f"Invalid region: Out-of-bounds {seqname}:{seq_len+1}-{seq_len+10} mismatch")
    
    print(f"Invalid region tests completed")
    return errors

def build_faigz_demo():
    """Build the faigz-rs demo binary."""
    print("Building faigz-rs demo binary...")
    rc, stdout, stderr = run_command("cargo build --bin faigz")
    if rc != 0:
        print(f"Build failed: {stderr}")
        return False
    return True

def main():
    fasta_file = "scerevisiae8.fa.gz"
    
    if not Path(fasta_file).exists():
        print(f"Error: {fasta_file} not found")
        sys.exit(1)
    
    # Build the demo binary
    if not build_faigz_demo():
        sys.exit(1)
    
    # Get sequence information
    print("Getting sequence information...")
    sequences = get_fasta_info(fasta_file)
    if not sequences:
        print("Error: Could not get sequence information")
        sys.exit(1)
    
    print(f"Found {len(sequences)} sequences:")
    for name, length in sequences[:5]:  # Show first 5
        print(f"  {name}: {length} bp")
    if len(sequences) > 5:
        print(f"  ... and {len(sequences) - 5} more")
    
    all_errors = []
    
    # Run tests
    all_errors.extend(test_random_regions(fasta_file, sequences, 200))
    all_errors.extend(test_edge_cases(fasta_file, sequences))
    all_errors.extend(test_invalid_regions(fasta_file, sequences))
    
    # Report results
    print("\n" + "="*60)
    print("TEST RESULTS")
    print("="*60)
    
    if all_errors:
        print(f"❌ {len(all_errors)} errors found:")
        for error in all_errors:
            print(f"  {error}")
        sys.exit(1)
    else:
        print("✅ All tests passed! faigz-rs random access matches samtools faidx behavior.")
        sys.exit(0)

if __name__ == "__main__":
    main()