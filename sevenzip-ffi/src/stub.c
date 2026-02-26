// =============================================================================
// CORE-FFX - Forensic File Explorer
// sevenzip-ffi Windows stub — provides linker symbols when the real C library
// (LZMA SDK 24.09) is not available. Every function returns an error code or
// NULL so callers get a clear "not supported" at runtime.
// =============================================================================

#include <string.h>
#include <stdint.h>

#define SZ_OK            0
#define SZ_ERROR_COMPRESS 5
#define SZ_NOT_IMPL      7

typedef int SevenZipErrorCode;

typedef void (*SevenZipProgressCallback)(uint64_t, uint64_t, void*);
typedef void (*SevenZipBytesProgressCallback)(uint64_t, uint64_t, uint64_t, uint64_t, const char*, void*);

typedef struct { int num_threads; uint64_t dict_size; int solid; const char* password; } SevenZipCompressOptions;
typedef struct { int num_threads; uint64_t dict_size; int solid; const char* password; uint64_t split_size; uint64_t chunk_size; const char* temp_dir; int delete_temp_on_error; } SevenZipStreamOptions;
typedef struct { int code; char message[512]; char file_context[256]; int64_t position; char suggestion[256]; } SevenZipErrorInfo;
typedef struct { void* entries; size_t count; } SevenZipList;

SevenZipErrorCode sevenzip_init(void) { return SZ_OK; }
void sevenzip_cleanup(void) {}

SevenZipErrorCode sevenzip_extract(const char* a, const char* b, const char* c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_extract_files(const char* a, const char* b, const char** c, const char* d, SevenZipProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_extract_archive(const char* a, const char* b, const char* c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_compress(const char* a, const char** b, int c, const char* d, SevenZipProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_create_archive(const char* a, const char** b, int c, const char* d, SevenZipProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_create_7z(const char* a, const char** b, int c, const SevenZipCompressOptions* d, SevenZipProgressCallback cb, void* e) { return SZ_NOT_IMPL; }

void sevenzip_stream_options_init(SevenZipStreamOptions* opts) { if(opts) memset(opts, 0, sizeof(*opts)); }
SevenZipErrorCode sevenzip_create_7z_streaming(const char* a, const char** b, int c, const SevenZipStreamOptions* d, SevenZipBytesProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_create_7z_true_streaming(const char* a, const char** b, int c, const SevenZipStreamOptions* d, SevenZipBytesProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_extract_streaming(const char* a, const char* b, const char* c, SevenZipBytesProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_compress_stream(const char* a, const char** b, int c, const SevenZipStreamOptions* d, SevenZipBytesProgressCallback cb, void* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_compress_resume(const char* a, const char* b, SevenZipBytesProgressCallback cb, void* c) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_list(const char* a, const char* b, SevenZipList** c) { if(c) *c = 0; return SZ_NOT_IMPL; }
void sevenzip_free_list(SevenZipList* l) {}
SevenZipErrorCode sevenzip_test_archive(const char* a, const char* b, SevenZipBytesProgressCallback cb, void* c) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_repair_archive(const char* a, const char* b, SevenZipProgressCallback cb, void* c) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_validate_archive(const char* a, SevenZipErrorInfo* b) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_compress_file(const char* a, const char* b, int c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_decompress_file(const char* a, const char* b, SevenZipProgressCallback cb, void* c) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_init_encryption(const char* a, uint8_t* b, uint8_t* c, uint32_t* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_init_decryption(const char* a, const uint8_t* b, size_t c, uint8_t* d, uint32_t* e) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_encrypt_data(uint32_t* a, const uint8_t* b, const uint8_t* c, size_t d, uint8_t* e, size_t* f) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_decrypt_data(uint32_t* a, const uint8_t* b, const uint8_t* c, size_t d, uint8_t* e, size_t* f) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_verify_password(const char* a, const uint8_t* b, size_t c, const uint8_t* d, size_t e, const uint8_t* f) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_decompress_lzma(const char* a, const char* b, SevenZipProgressCallback cb, void* c) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_decompress_lzma2(const char* a, const char* b, SevenZipProgressCallback cb, void* c) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_compress_lzma(const char* a, const char* b, int c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_compress_lzma2(const char* a, const char* b, int c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_create_multivolume_7z(const char* a, const char** b, int c, uint64_t d, const SevenZipCompressOptions* e, SevenZipProgressCallback cb, void* f) { return SZ_NOT_IMPL; }
SevenZipErrorCode sevenzip_extract_split_archive(const char* a, const char* b, const char* c, SevenZipProgressCallback cb, void* d) { return SZ_NOT_IMPL; }

SevenZipErrorCode sevenzip_get_last_error(SevenZipErrorInfo* e) { return SZ_OK; }
void sevenzip_clear_last_error(void) {}
const char* sevenzip_get_error_string(SevenZipErrorCode c) { return "7z FFI stub: not available on this platform"; }
const char* sevenzip_get_version(void) { return "stub-1.0.0"; }

SevenZipErrorCode sevenzip_generate_forensic_manifest(const char* a, const char** b, const char* c, SevenZipBytesProgressCallback cb, void* d) { return SZ_NOT_IMPL; }
