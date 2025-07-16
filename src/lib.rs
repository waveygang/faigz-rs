//! # faigz-rs
//!
//! A Rust wrapper for the faigz reentrant FASTA/FASTQ index library.
//!
//! This library provides thread-safe, reentrant access to FASTA and FASTQ files
//! using a shared index structure that can be safely accessed from multiple threads.
//!
//! ## Features
//!
//! - Thread-safe FASTA/FASTQ file access
//! - Reentrant index structure for multi-threaded applications
//! - Support for both FASTA and FASTQ formats
//! - Efficient memory usage with shared metadata
//! - Compatible with bgzip-compressed files
//!
//! ## Example
//!
//! ```rust,no_run
//! use faigz_rs::{FastaIndex, FastaReader, FastaFormat};
//!
//! // Load the index metadata once
//! let index = FastaIndex::new("genome.fa", FastaFormat::Fasta)?;
//!
//! // Create readers for each thread
//! let reader = FastaReader::new(&index)?;
//!
//! // Fetch sequence data
//! let sequence = reader.fetch_seq("chr1", 1000, 1100)?;
//! println!("Sequence: {}", sequence);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};
use std::sync::Arc;
use thiserror::Error;

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Constants from htslib faidx.h
const FAI_CREATE: c_int = 0x01;

/// Error types for FASTA operations
#[derive(Error, Debug)]
pub enum FastaError {
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    #[error("Failed to load index: {0}")]
    IndexLoadError(String),
    #[error("Failed to create reader")]
    ReaderCreationError,
    #[error("Sequence not found: {0}")]
    SequenceNotFound(String),
    #[error("Invalid region: {0}")]
    InvalidRegion(String),
    #[error("Memory allocation failed")]
    MemoryError,
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Quality data not available (FASTA format)")]
    QualityNotAvailable,
}

/// Result type for FASTA operations
pub type FastaResult<T> = Result<T, FastaError>;

/// Format options for FASTA/FASTQ files
#[derive(Debug, Clone, Copy)]
pub enum FastaFormat {
    /// FASTA format
    Fasta,
    /// FASTQ format
    Fastq,
}

impl From<FastaFormat> for fai_format_options {
    fn from(format: FastaFormat) -> Self {
        match format {
            FastaFormat::Fasta => FAI_FASTA,
            FastaFormat::Fastq => FAI_FASTQ,
        }
    }
}

/// Shared FASTA index metadata
///
/// This structure holds the shared metadata for a FASTA/FASTQ file that can be
/// safely accessed from multiple threads. It uses reference counting to manage
/// the lifetime of the underlying C structure.
pub struct FastaIndex {
    meta: *mut faidx_meta_t,
}

impl std::fmt::Debug for FastaIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastaIndex")
            .field("num_sequences", &self.num_sequences())
            .field("sequence_names", &self.sequence_names())
            .finish()
    }
}

impl FastaIndex {
    /// Create a new FASTA index from a file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the FASTA/FASTQ file
    /// * `format` - Format of the file (FASTA or FASTQ)
    ///
    /// # Returns
    ///
    /// A new `FastaIndex` instance or an error if the file cannot be loaded
    pub fn new(path: &str, format: FastaFormat) -> FastaResult<Self> {
        let c_path = CString::new(path).map_err(|_| FastaError::InvalidPath(path.to_string()))?;

        let meta = unsafe { faidx_meta_load(c_path.as_ptr(), format.into(), FAI_CREATE) };

        if meta.is_null() {
            return Err(FastaError::IndexLoadError(path.to_string()));
        }

        Ok(FastaIndex { meta })
    }

    /// Get the number of sequences in the index
    pub fn num_sequences(&self) -> usize {
        unsafe { faidx_meta_nseq(self.meta) as usize }
    }

    /// Get the name of the sequence at the given index
    pub fn sequence_name(&self, index: usize) -> Option<String> {
        let name_ptr = unsafe { faidx_meta_iseq(self.meta, index as c_int) };
        if name_ptr.is_null() {
            None
        } else {
            let c_str = unsafe { CStr::from_ptr(name_ptr) };
            Some(c_str.to_string_lossy().to_string())
        }
    }

    /// Get the length of the specified sequence
    pub fn sequence_length(&self, name: &str) -> Option<i64> {
        let c_name = CString::new(name).ok()?;
        let length = unsafe { faidx_meta_seq_len(self.meta, c_name.as_ptr()) };
        if length < 0 {
            None
        } else {
            Some(length)
        }
    }

    /// Check if the index contains the specified sequence
    pub fn has_sequence(&self, name: &str) -> bool {
        let c_name = CString::new(name).unwrap_or_else(|_| CString::new("").unwrap());
        unsafe { faidx_meta_has_seq(self.meta, c_name.as_ptr()) != 0 }
    }

    /// Get all sequence names in the index
    pub fn sequence_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let n = self.num_sequences();
        for i in 0..n {
            if let Some(name) = self.sequence_name(i) {
                names.push(name);
            }
        }
        names
    }
}

impl Clone for FastaIndex {
    fn clone(&self) -> Self {
        let meta = unsafe { faidx_meta_ref(self.meta) };
        FastaIndex { meta }
    }
}

impl Drop for FastaIndex {
    fn drop(&mut self) {
        unsafe {
            faidx_meta_destroy(self.meta);
        }
    }
}

unsafe impl Send for FastaIndex {}
unsafe impl Sync for FastaIndex {}

/// FASTA reader for accessing sequences
///
/// This structure provides thread-safe access to FASTA/FASTQ sequences using
/// a shared index. Each reader maintains its own file handle but shares the
/// index metadata.
pub struct FastaReader {
    reader: *mut faidx_reader_t,
    _index: Arc<FastaIndex>, // Keep index alive
}

impl FastaReader {
    /// Create a new FASTA reader from an index
    ///
    /// # Arguments
    ///
    /// * `index` - Shared FASTA index
    ///
    /// # Returns
    ///
    /// A new `FastaReader` instance or an error if the reader cannot be created
    pub fn new(index: &FastaIndex) -> FastaResult<Self> {
        let reader = unsafe { faidx_reader_create(index.meta) };

        if reader.is_null() {
            return Err(FastaError::ReaderCreationError);
        }

        Ok(FastaReader {
            reader,
            _index: Arc::new(index.clone()),
        })
    }

    /// Fetch a sequence from the specified region
    ///
    /// # Arguments
    ///
    /// * `seqname` - Name of the sequence
    /// * `start` - Start position (0-based, inclusive)
    /// * `end` - End position (0-based, exclusive)
    ///
    /// # Returns
    ///
    /// The sequence string or an error if the sequence cannot be fetched
    pub fn fetch_seq(&self, seqname: &str, start: i64, end: i64) -> FastaResult<String> {
        let c_seqname =
            CString::new(seqname).map_err(|_| FastaError::SequenceNotFound(seqname.to_string()))?;

        let mut len: i64 = 0;
        let seq_ptr = unsafe {
            faidx_reader_fetch_seq(self.reader, c_seqname.as_ptr(), start, end - 1, &mut len)
        };

        if seq_ptr.is_null() {
            return Err(FastaError::SequenceNotFound(seqname.to_string()));
        }

        let c_str = unsafe { CStr::from_ptr(seq_ptr) };
        let result = c_str.to_string_lossy().to_string();

        unsafe {
            libc::free(seq_ptr as *mut c_void);
        }

        Ok(result)
    }

    /// Fetch the entire sequence
    ///
    /// # Arguments
    ///
    /// * `seqname` - Name of the sequence
    ///
    /// # Returns
    ///
    /// The complete sequence string or an error if the sequence cannot be fetched
    pub fn fetch_seq_all(&self, seqname: &str) -> FastaResult<String> {
        let length = self
            ._index
            .sequence_length(seqname)
            .ok_or_else(|| FastaError::SequenceNotFound(seqname.to_string()))?;

        self.fetch_seq(seqname, 0, length)
    }

    /// Fetch quality scores for the specified region (FASTQ only)
    ///
    /// # Arguments
    ///
    /// * `seqname` - Name of the sequence
    /// * `start` - Start position (0-based, inclusive)
    /// * `end` - End position (0-based, exclusive)
    ///
    /// # Returns
    ///
    /// The quality string or an error if the quality cannot be fetched
    pub fn fetch_qual(&self, seqname: &str, start: i64, end: i64) -> FastaResult<String> {
        let c_seqname =
            CString::new(seqname).map_err(|_| FastaError::SequenceNotFound(seqname.to_string()))?;

        let mut len: i64 = 0;
        let qual_ptr = unsafe {
            faidx_reader_fetch_qual(self.reader, c_seqname.as_ptr(), start, end - 1, &mut len)
        };

        if qual_ptr.is_null() {
            return Err(FastaError::QualityNotAvailable);
        }

        let c_str = unsafe { CStr::from_ptr(qual_ptr) };
        let result = c_str.to_string_lossy().to_string();

        unsafe {
            libc::free(qual_ptr as *mut c_void);
        }

        Ok(result)
    }

    /// Parse a region string (e.g., "chr1:1000-2000") and fetch the sequence
    ///
    /// # Arguments
    ///
    /// * `region` - Region string in format "seqname:start-end"
    ///
    /// # Returns
    ///
    /// The sequence string or an error if the region cannot be parsed or fetched
    pub fn fetch_region(&self, region: &str) -> FastaResult<String> {
        // Simple region parsing - you might want to use the C function for more complex cases
        if let Some(colon_pos) = region.find(':') {
            let seqname = &region[..colon_pos];
            let range_part = &region[colon_pos + 1..];

            if let Some(dash_pos) = range_part.find('-') {
                let start_str = &range_part[..dash_pos];
                let end_str = &range_part[dash_pos + 1..];

                let start: i64 = start_str
                    .parse()
                    .map_err(|_| FastaError::InvalidRegion(region.to_string()))?;
                let end: i64 = end_str
                    .parse()
                    .map_err(|_| FastaError::InvalidRegion(region.to_string()))?;

                // Convert from 1-based to 0-based coordinates
                self.fetch_seq(seqname, start - 1, end)
            } else {
                Err(FastaError::InvalidRegion(region.to_string()))
            }
        } else {
            // No colon, assume it's just a sequence name
            self.fetch_seq_all(region)
        }
    }
}

impl Drop for FastaReader {
    fn drop(&mut self) {
        unsafe {
            faidx_reader_destroy(self.reader);
        }
    }
}

unsafe impl Send for FastaReader {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_fasta() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, ">seq1").unwrap();
        writeln!(file, "ATCGATCGATCGATCG").unwrap();
        writeln!(file, ">seq2").unwrap();
        writeln!(file, "GCTAGCTAGCTAGCTA").unwrap();
        writeln!(file, "AAAAAAAAAAAAAAAA").unwrap();
        file
    }

    #[test]
    fn test_index_creation() {
        let mut fasta_file = create_test_fasta();
        fasta_file.flush().unwrap(); // Ensure data is written
        let path = fasta_file.path().to_str().unwrap();

        let index = FastaIndex::new(path, FastaFormat::Fasta).unwrap();
        assert!(index.num_sequences() > 0);
    }

    #[test]
    fn test_error_handling() {
        let result = FastaIndex::new("/nonexistent/file.fa", FastaFormat::Fasta);
        assert!(result.is_err());

        match result.unwrap_err() {
            FastaError::IndexLoadError(_) => (),
            _ => panic!("Expected IndexLoadError"),
        }
    }
}
