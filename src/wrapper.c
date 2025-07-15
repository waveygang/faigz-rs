// Stub implementation for compilation when htslib is not available
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Mock structures for compilation
typedef struct {
    int n;
    char **names;
    int *lengths;
} faidx_meta_t;

typedef struct {
    faidx_meta_t *meta;
} faidx_reader_t;

typedef int fai_format_options;
#define FAI_FASTA 0
#define FAI_FASTQ 1

// Stub function implementations
faidx_meta_t *faidx_meta_load(const char *filename, fai_format_options format, int flags) {
    // Stub implementation - always fails
    (void)filename;  // Mark as intentionally unused
    (void)format;    // Mark as intentionally unused
    (void)flags;     // Mark as intentionally unused
    return NULL;
}

faidx_meta_t *faidx_meta_ref(faidx_meta_t *meta) {
    return meta;
}

void faidx_meta_destroy(faidx_meta_t *meta) {
    if (meta) {
        free(meta);
    }
}

int faidx_meta_nseq(const faidx_meta_t *meta) {
    return meta ? meta->n : 0;
}

const char *faidx_meta_iseq(const faidx_meta_t *meta, int i) {
    return (meta && i >= 0 && i < meta->n) ? meta->names[i] : NULL;
}

long faidx_meta_seq_len(const faidx_meta_t *meta, const char *seq) {
    // Stub implementation
    (void)meta;  // Mark as intentionally unused
    (void)seq;   // Mark as intentionally unused
    return -1;
}

int faidx_meta_has_seq(const faidx_meta_t *meta, const char *seq) {
    // Stub implementation
    (void)meta;  // Mark as intentionally unused
    (void)seq;   // Mark as intentionally unused
    return 0;
}

faidx_reader_t *faidx_reader_create(faidx_meta_t *meta) {
    // Stub implementation
    (void)meta;  // Mark as intentionally unused
    return NULL;
}

void faidx_reader_destroy(faidx_reader_t *reader) {
    if (reader) {
        free(reader);
    }
}

char *faidx_reader_fetch_seq(faidx_reader_t *reader, const char *c_name, long p_beg_i, long p_end_i, long *len) {
    // Stub implementation - always fails
    (void)reader;   // Mark as intentionally unused
    (void)c_name;   // Mark as intentionally unused
    (void)p_beg_i;  // Mark as intentionally unused
    (void)p_end_i;  // Mark as intentionally unused
    if (len) *len = -1;
    return NULL;
}

char *faidx_reader_fetch_qual(faidx_reader_t *reader, const char *c_name, long p_beg_i, long p_end_i, long *len) {
    // Stub implementation - always fails
    (void)reader;   // Mark as intentionally unused
    (void)c_name;   // Mark as intentionally unused
    (void)p_beg_i;  // Mark as intentionally unused
    (void)p_end_i;  // Mark as intentionally unused
    if (len) *len = -1;
    return NULL;
}