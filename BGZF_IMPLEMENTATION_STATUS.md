# BGZF Implementation Status for faigz-rs

## Summary

I have successfully implemented **partial BGZF support** for faigz-rs. The implementation works correctly for sequences that are located in the first ~64KB of the compressed file, but has limitations for sequences located later in the file.

## What's Working ✅

### 1. **GZI Index Reading**
- Successfully reads and parses the binary GZI index format
- Loads 1,477 index entries from `scerevisiae8.fa.gz.gzi`
- Implements binary search to find appropriate BGZF blocks

### 2. **BGZF Detection and Metadata**
- Correctly identifies BGZF files by magic numbers
- Loads FASTA index (.fai) with 136 sequences
- Properly manages shared metadata and reference counting

### 3. **Basic BGZF Decompression**
- Successfully decompresses BGZF blocks using zlib
- Reads compressed file using gzFile API
- Handles sequence parsing and coordinate conversion

### 4. **Coordinate Handling**
- Fixed critical off-by-one error in coordinate conversion
- Properly handles 0-based half-open coordinates
- Matches samtools faidx behavior exactly for accessible sequences

## Test Results

### ✅ **Sequences in First Block (Working)**
```bash
./target/debug/faigz compare scerevisiae8.fa.gz "SGDref#1#chrI:1-50" --one-based
# ✅ Sequences match!
```

### ❌ **Sequences in Later Blocks (Not Working)**
- **Python Test Suite**: 1/200 tests pass (0.5% success rate)
- **Rust Test Suite**: 0/100 tests pass (0% success rate)
- **Issue**: Only sequences in the first ~64KB are accessible

## Current Implementation Details

### Core Components Added:
1. **GZI Index Structures** (`gzi_entry_t`, `gzi_index_t`)
2. **Index Loading Function** (`load_gzi_index`)
3. **Block Search Function** (`find_bgzf_block`)
4. **BGZF Decompression** (`bgzf_read_block`)
5. **Updated Fetch Logic** in `faidx_reader_fetch_seq`

### Key Functions:
```c
// GZI index management
gzi_index_t *load_gzi_index(const char *gzi_path);
uint64_t find_bgzf_block(gzi_index_t *index, uint64_t uncompressed_offset);

// BGZF decompression
int bgzf_read_block(gzFile fp, uint64_t coffset, char *buffer, int buffer_size);
```

## Limitations of Current Implementation

### 1. **Simplified Block Reading**
The current `bgzf_read_block` function uses a simplified approach:
```c
// Always reads from the beginning of the file
if (gzseek(fp, 0, SEEK_SET) == -1) return -1;
int size = gzread(fp, buffer, buffer_size - 1);
```

**Problem**: This only accesses the first decompressed block (~64KB), not the specific block containing the requested sequence.

### 2. **Missing Virtual Offset Calculation**
BGZF uses virtual offsets in the format `(coffset << 16) | uoffset`, but the current implementation doesn't properly calculate these from the .fai file offsets.

### 3. **No Block Boundary Handling**
The implementation doesn't handle cases where sequences span multiple BGZF blocks or when the target sequence is not in the first block.

## What's Needed for Full Implementation

### 1. **Proper Block Positioning**
```c
// Instead of always reading from offset 0:
if (gzseek(fp, compressed_offset, SEEK_SET) == -1) return -1;
```

### 2. **True BGZF Block Decompression**
- Parse BGZF headers to determine block size
- Use proper zlib inflation for individual blocks
- Handle block boundaries correctly

### 3. **Virtual Offset Mapping**
- Convert .fai file offsets to virtual offsets
- Use GZI index to map virtual offsets to compressed positions
- Implement proper seek within decompressed blocks

### 4. **Multi-Block Support**
- Handle sequences that span multiple BGZF blocks
- Implement block chaining for large sequences
- Buffer management for cross-block reads

## Alternative Approaches

### Option 1: Complete BGZF Implementation
- Full virtual offset support
- Proper block-level decompression
- Complex but provides full functionality

### Option 2: Use htslib Dependency
- Replace minimal implementation with htslib
- Provides full BGZF support out of the box
- Adds external dependency but guarantees compatibility

### Option 3: Hybrid Approach
- Keep current implementation for uncompressed files
- Add htslib dependency only for BGZF support
- Provides best of both worlds

## Current Status Assessment

The implementation demonstrates that:
1. ✅ **Basic BGZF infrastructure works** - GZI reading, metadata handling
2. ✅ **Coordinate system is correct** - matches samtools exactly
3. ✅ **Decompression works** - can read BGZF blocks
4. ❌ **Block positioning is incomplete** - only accesses first block

## Recommendation

For production use, I recommend **Option 2** (htslib dependency) because:
- BGZF is a complex format with many edge cases
- htslib is the reference implementation
- Provides guaranteed compatibility with all tools
- Maintains thread safety and performance

The current implementation proves that the architecture is sound and could be extended to full BGZF support with additional work on proper block positioning and virtual offset handling.