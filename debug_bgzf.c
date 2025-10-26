#include "faigz_minimal.h"
#include <stdio.h>

int main() {
    const char *fasta_file = "scerevisiae8.fa.gz";
    
    printf("Loading index...\n");
    faidx_meta_t *meta = faidx_meta_load(fasta_file, FAI_FASTA, FAI_CREATE);
    if (!meta) {
        printf("Failed to load metadata\n");
        return 1;
    }
    
    printf("Is BGZF: %s\n", meta->is_bgzf ? "yes" : "no");
    printf("GZI index loaded: %s\n", meta->gzi_index ? "yes" : "no");
    
    if (meta->gzi_index) {
        printf("GZI index entries: %d\n", meta->gzi_index->n_entries);
        printf("First few entries:\n");
        for (int i = 0; i < 5 && i < meta->gzi_index->n_entries; i++) {
            printf("  %d: compressed=%llu, uncompressed=%llu\n", i, 
                   meta->gzi_index->entries[i].compressed_offset,
                   meta->gzi_index->entries[i].uncompressed_offset);
        }
    }
    
    printf("Number of sequences: %d\n", meta->n);
    if (meta->n > 0) {
        printf("First sequence: '%s'\n", meta->name[0]);
        
        // Look up the sequence in the hash table
        faidx1_t *entry = faidx_meta_get_entry(meta, meta->name[0]);
        if (entry) {
            printf("Found entry - seq_offset: %llu, len: %llu\n", 
                   entry->seq_offset, entry->len);
        } else {
            printf("Entry not found in hash table\n");
        }
    }
    
    faidx_meta_destroy(meta);
    return 0;
}