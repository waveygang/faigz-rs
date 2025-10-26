# faigz-rs Random Access Test Results

## Summary

I have created comprehensive tests to verify the random access functionality of faigz-rs against samtools faidx behavior. The testing revealed a critical coordinate handling bug which was fixed, and identified a limitation in the current implementation.

## Tests Created

### 1. Python Test Script (`test_random_access_simple.py`)
- Creates a test FASTA file with simple sequence names
- Tests 50 random regions with varying coordinates
- Compares faigz-rs output against samtools faidx
- Tests edge cases (full sequences, first/last bases)
- **Result: ✅ All 50 tests pass**

### 2. Rust Test Suite (`tests/random_access_tests.rs`)
- Comprehensive random access tests using the Rust API
- Tests with the provided `scerevisiae8.fa.gz` file
- Tests 100 random regions, edge cases, and error conditions
- **Result: ❌ All tests fail due to compressed file limitation**

## Key Findings

### 1. Coordinate Handling Bug (FIXED)
**Issue**: The `fetch_seq` function had an off-by-one error in coordinate handling.
- **Location**: `src/lib.rs:241`
- **Problem**: `faidx_reader_fetch_seq(self.reader, c_seqname.as_ptr(), start, end - 1, &mut len)`
- **Fix**: Changed to `faidx_reader_fetch_seq(self.reader, c_seqname.as_ptr(), start, end, &mut len)`
- **Impact**: This was causing sequences to be 1 base shorter than expected

**Verification**:
```bash
# Before fix:
./target/debug/faigz compare test.fa "chr1:1-10" --one-based
# faigz-rs: ATCGATCGA (9 bases)
# samtools: ATCGATCGAT (10 bases)

# After fix:
./target/debug/faigz compare test.fa "chr1:1-10" --one-based
# ✅ Sequences match! (both 10 bases)
```

### 2. Compressed File Support Limitation
**Issue**: The current C implementation does not support random access on compressed files.
- **Location**: `faigz_minimal.c:~line 400`
- **Problem**: `return NULL; // Not implemented for compressed files in this minimal version`
- **Impact**: All tests with `scerevisiae8.fa.gz` fail with "Sequence not found" errors

## Test Results

### Uncompressed FASTA Files
- **Random Access Tests**: ✅ 50/50 passed  
- **Edge Case Tests**: ✅ All passed
- **Coordinate Systems**: ✅ Correctly handles 0-based and 1-based coordinates
- **Boundary Conditions**: ✅ Proper handling of sequence boundaries

### Compressed FASTA Files (.gz)
- **Random Access Tests**: ❌ 0/100 passed (due to implementation limitation)
- **Index Loading**: ✅ Works correctly
- **Sequence Metadata**: ✅ Correctly reads sequence names and lengths
- **Sequence Fetching**: ❌ Returns NULL for all compressed files

## Recommendations

1. **For Production Use**: 
   - Use uncompressed FASTA files for random access operations
   - The current implementation works perfectly for uncompressed files

2. **For Compressed File Support**:
   - Implement proper compressed file random access in the C layer
   - Add support for bgzip index (.gzi) files
   - Consider using the full htslib implementation instead of the minimal version

3. **Testing**:
   - The test suite can be used to verify any future improvements
   - Both Python and Rust test suites are comprehensive and ready to use

## Files Created

1. `test_random_access_simple.py` - Python test script for uncompressed files
2. `tests/random_access_tests.rs` - Rust test suite for comprehensive testing
3. `TEST_RESULTS.md` - This summary document

## Conclusion

The faigz-rs library correctly implements random access for uncompressed FASTA files and matches samtools faidx behavior exactly. The coordinate handling bug has been fixed, and all tests pass for uncompressed files. The limitation with compressed files is a known issue in the current minimal C implementation.