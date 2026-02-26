// =============================================================================
// CORE-FFX - Forensic File Explorer
// libewf-ffi Windows stub — provides linker symbols when the real libewf
// C library is not available. Every function returns -1 (error) so callers
// get a clear "not supported" at runtime.
// =============================================================================

#include <stdint.h>
#include <string.h>

typedef intptr_t libewf_handle_t;
typedef intptr_t libewf_error_t;
typedef intptr_t libewf_file_entry_t;
typedef intptr_t libewf_data_chunk_t;

static const char* STUB_VERSION = "stub-0.0.0";

const char* libewf_get_version(void) { return STUB_VERSION; }
int libewf_get_access_flags_read(void) { return 0x01; }
int libewf_get_access_flags_read_write(void) { return 0x03; }
int libewf_get_access_flags_write(void) { return 0x02; }

void libewf_error_free(libewf_error_t** e) { if(e) *e = 0; }
int libewf_error_sprint(libewf_error_t* e, char* s, size_t sz) { if(s && sz > 0) { strncpy(s, "libewf stub: not available", sz-1); s[sz-1] = 0; } return -1; }
int libewf_error_backtrace_sprint(libewf_error_t* e, char* s, size_t sz) { return libewf_error_sprint(e, s, sz); }

int libewf_handle_initialize(libewf_handle_t** h, libewf_error_t** e) { return -1; }
int libewf_handle_free(libewf_handle_t** h, libewf_error_t** e) { return -1; }
int libewf_handle_signal_abort(libewf_handle_t* h, libewf_error_t** e) { return -1; }

int libewf_glob(const char* f, size_t l, int t, char*** fl, int* n, libewf_error_t** e) { return -1; }
int libewf_glob_free(char** fl, int n, libewf_error_t** e) { return -1; }
int libewf_handle_open(libewf_handle_t* h, char* const* fl, int n, int a, libewf_error_t** e) { return -1; }
int libewf_handle_close(libewf_handle_t* h, libewf_error_t** e) { return -1; }

int64_t libewf_handle_read_buffer(libewf_handle_t* h, void* b, size_t s, libewf_error_t** e) { return -1; }
int64_t libewf_handle_read_buffer_at_offset(libewf_handle_t* h, void* b, size_t s, int64_t o, libewf_error_t** e) { return -1; }

int64_t libewf_handle_write_buffer(libewf_handle_t* h, const void* b, size_t s, libewf_error_t** e) { return -1; }
int64_t libewf_handle_write_buffer_at_offset(libewf_handle_t* h, const void* b, size_t s, int64_t o, libewf_error_t** e) { return -1; }

int64_t libewf_handle_write_finalize(libewf_handle_t* h, libewf_error_t** e) { return -1; }

int64_t libewf_handle_seek_offset(libewf_handle_t* h, int64_t o, int w, libewf_error_t** e) { return -1; }

int libewf_handle_set_segment_filename(libewf_handle_t* h, const uint8_t* n, size_t l, libewf_error_t** e) { return -1; }
int libewf_handle_get_maximum_segment_size(libewf_handle_t* h, size_t* s, libewf_error_t** e) { return -1; }
int libewf_handle_set_maximum_segment_size(libewf_handle_t* h, size_t s, libewf_error_t** e) { return -1; }

int libewf_handle_get_sectors_per_chunk(libewf_handle_t* h, uint32_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_sectors_per_chunk(libewf_handle_t* h, uint32_t v, libewf_error_t** e) { return -1; }
int libewf_handle_get_bytes_per_sector(libewf_handle_t* h, uint32_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_bytes_per_sector(libewf_handle_t* h, uint32_t v, libewf_error_t** e) { return -1; }
int libewf_handle_get_media_size(libewf_handle_t* h, size_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_media_size(libewf_handle_t* h, size_t v, libewf_error_t** e) { return -1; }
int libewf_handle_get_media_type(libewf_handle_t* h, uint8_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_media_type(libewf_handle_t* h, uint8_t v, libewf_error_t** e) { return -1; }
int libewf_handle_get_media_flags(libewf_handle_t* h, uint8_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_media_flags(libewf_handle_t* h, uint8_t v, libewf_error_t** e) { return -1; }
int libewf_handle_get_format(libewf_handle_t* h, uint8_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_format(libewf_handle_t* h, uint8_t v, libewf_error_t** e) { return -1; }

int libewf_handle_get_compression_values(libewf_handle_t* h, int8_t* l, uint8_t* f, libewf_error_t** e) { return -1; }
int libewf_handle_set_compression_values(libewf_handle_t* h, int8_t l, uint8_t f, libewf_error_t** e) { return -1; }

int libewf_handle_get_compression_method(libewf_handle_t* h, uint16_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_compression_method(libewf_handle_t* h, uint16_t v, libewf_error_t** e) { return -1; }

int libewf_handle_get_data_chunk(libewf_handle_t* h, libewf_data_chunk_t** c, libewf_error_t** e) { return -1; }
int libewf_handle_read_data_chunk(libewf_handle_t* h, libewf_data_chunk_t* c, libewf_error_t** e) { return -1; }
int libewf_handle_write_data_chunk(libewf_handle_t* h, libewf_data_chunk_t* c, libewf_error_t** e) { return -1; }
int libewf_data_chunk_free(libewf_data_chunk_t** c, libewf_error_t** e) { return -1; }

int libewf_handle_get_offset(libewf_handle_t* h, int64_t* o, libewf_error_t** e) { return -1; }

int libewf_handle_segment_files_corrupted(libewf_handle_t* h, libewf_error_t** e) { return -1; }
int libewf_handle_segment_files_encrypted(libewf_handle_t* h, libewf_error_t** e) { return -1; }
int libewf_handle_get_segment_file_version(libewf_handle_t* h, uint8_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_get_number_of_chunks_written(libewf_handle_t* h, uint32_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_set_read_zero_chunk_on_error(libewf_handle_t* h, uint8_t v, libewf_error_t** e) { return -1; }

int libewf_handle_copy_media_values(libewf_handle_t* d, libewf_handle_t* s, libewf_error_t** e) { return -1; }
int libewf_handle_copy_header_values(libewf_handle_t* d, libewf_handle_t* s, libewf_error_t** e) { return -1; }

int libewf_handle_get_number_of_header_values(libewf_handle_t* h, uint32_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_get_header_value_identifier_size(libewf_handle_t* h, uint32_t i, size_t* s, libewf_error_t** e) { return -1; }
int libewf_handle_get_header_value_identifier(libewf_handle_t* h, uint32_t i, uint8_t* v, size_t s, libewf_error_t** e) { return -1; }
int libewf_handle_get_utf8_header_value_size(libewf_handle_t* h, const uint8_t* id, size_t il, size_t* vs, libewf_error_t** e) { return -1; }
int libewf_handle_get_utf8_header_value(libewf_handle_t* h, const uint8_t* id, size_t il, uint8_t* v, size_t vs, libewf_error_t** e) { return -1; }
int libewf_handle_set_utf8_header_value(libewf_handle_t* h, const uint8_t* id, size_t il, const uint8_t* v, size_t vs, libewf_error_t** e) { return -1; }

int libewf_handle_get_number_of_hash_values(libewf_handle_t* h, uint32_t* v, libewf_error_t** e) { return -1; }
int libewf_handle_get_md5_hash(libewf_handle_t* h, uint8_t* v, size_t s, libewf_error_t** e) { return -1; }
int libewf_handle_set_md5_hash(libewf_handle_t* h, const uint8_t* v, size_t s, libewf_error_t** e) { return -1; }
int libewf_handle_get_sha1_hash(libewf_handle_t* h, uint8_t* v, size_t s, libewf_error_t** e) { return -1; }
int libewf_handle_set_sha1_hash(libewf_handle_t* h, const uint8_t* v, size_t s, libewf_error_t** e) { return -1; }
int libewf_handle_set_utf8_hash_value(libewf_handle_t* h, const uint8_t* id, size_t il, const uint8_t* v, size_t vs, libewf_error_t** e) { return -1; }
