#!/usr/bin/env python3

import subprocess

def run_command(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True, shell=True)
    return result.returncode, result.stdout, result.stderr

# Test the first sequence
fasta_file = "scerevisiae8.fa.gz"

print("=== Testing sequence name extraction ===")
rc, stdout, stderr = run_command(f"./target/debug/faigz info {fasta_file} | head -5")
print(f"Info output:\n{stdout}")

print("\n=== Testing samtools with first sequence ===")
rc, stdout, stderr = run_command(f"samtools faidx {fasta_file} 'SGDref#1#chrI:1-50'")
print(f"Samtools output:\n{stdout}")

print("\n=== Testing faigz with first sequence ===")
rc, stdout, stderr = run_command(f"./target/debug/faigz extract {fasta_file} 'SGDref#1#chrI:1-50'")
print(f"Return code: {rc}")
print(f"Faigz output:\n{stdout}")
print(f"Faigz stderr:\n{stderr}")

print("\n=== Testing faigz with 1-based coordinates ===")
rc, stdout, stderr = run_command(f"./target/debug/faigz extract --one-based {fasta_file} 'SGDref#1#chrI:1-50'")
print(f"Return code: {rc}")
print(f"Faigz output:\n{stdout}")
print(f"Faigz stderr:\n{stderr}")

print("\n=== Testing faigz with full sequence ===")
rc, stdout, stderr = run_command(f"./target/debug/faigz extract {fasta_file} 'SGDref#1#chrI'")
print(f"Return code: {rc}")
print(f"Faigz output (first 200 chars):\n{stdout[:200]}")
print(f"Faigz stderr:\n{stderr}")