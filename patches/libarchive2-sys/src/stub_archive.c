/* Auto-generated stub library for cross-compilation */
/* Archive operations will return errors at runtime */

#include <stddef.h>
#include <stdint.h>

#define ARCHIVE_FATAL -30
#define ARCHIVE_OK 0

struct archive;
struct archive_entry;
struct archive_entry_linkresolver;
struct archive_match;
struct archive_read_disk;

int archive_version_number() { return 0; }
void* archive_version_string() { return NULL; }
void* archive_version_details() { return NULL; }
void* archive_zlib_version() { return NULL; }
void* archive_liblzma_version() { return NULL; }
void* archive_bzlib_version() { return NULL; }
void* archive_liblz4_version() { return NULL; }
void* archive_libzstd_version() { return NULL; }
void* archive_liblzo2_version() { return NULL; }
void* archive_libexpat_version() { return NULL; }
void* archive_libbsdxml_version() { return NULL; }
void* archive_libxml2_version() { return NULL; }
void* archive_mbedtls_version() { return NULL; }
void* archive_nettle_version() { return NULL; }
void* archive_openssl_version() { return NULL; }
void* archive_libmd_version() { return NULL; }
void* archive_commoncrypto_version() { return NULL; }
void* archive_cng_version() { return NULL; }
void* archive_wincrypt_version() { return NULL; }
void* archive_librichacl_version() { return NULL; }
void* archive_libacl_version() { return NULL; }
void* archive_libattr_version() { return NULL; }
void* archive_libiconv_version() { return NULL; }
void* archive_libpcre_version() { return NULL; }
void* archive_libpcre2_version() { return NULL; }
void* archive_read_new() { return NULL; }
int archive_read_support_compression_all() { return ARCHIVE_FATAL; }
int archive_read_support_compression_bzip2() { return ARCHIVE_FATAL; }
int archive_read_support_compression_compress() { return ARCHIVE_FATAL; }
int archive_read_support_compression_gzip() { return ARCHIVE_FATAL; }
int archive_read_support_compression_lzip() { return ARCHIVE_FATAL; }
int archive_read_support_compression_lzma() { return ARCHIVE_FATAL; }
int archive_read_support_compression_none() { return ARCHIVE_FATAL; }
int archive_read_support_compression_program() { return ARCHIVE_FATAL; }
int archive_read_support_compression_program_signature() { return ARCHIVE_FATAL; }
int archive_read_support_compression_rpm() { return ARCHIVE_FATAL; }
int archive_read_support_compression_uu() { return ARCHIVE_FATAL; }
int archive_read_support_compression_xz() { return ARCHIVE_FATAL; }
int archive_read_support_filter_all() { return ARCHIVE_FATAL; }
int archive_read_support_filter_by_code() { return ARCHIVE_FATAL; }
int archive_read_support_filter_bzip2() { return ARCHIVE_FATAL; }
int archive_read_support_filter_compress() { return ARCHIVE_FATAL; }
int archive_read_support_filter_gzip() { return ARCHIVE_FATAL; }
int archive_read_support_filter_grzip() { return ARCHIVE_FATAL; }
int archive_read_support_filter_lrzip() { return ARCHIVE_FATAL; }
int archive_read_support_filter_lz4() { return ARCHIVE_FATAL; }
int archive_read_support_filter_lzip() { return ARCHIVE_FATAL; }
int archive_read_support_filter_lzma() { return ARCHIVE_FATAL; }
int archive_read_support_filter_lzop() { return ARCHIVE_FATAL; }
int archive_read_support_filter_none() { return ARCHIVE_FATAL; }
int archive_read_support_filter_program() { return ARCHIVE_FATAL; }
int archive_read_support_filter_program_signature() { return ARCHIVE_FATAL; }
int archive_read_support_filter_rpm() { return ARCHIVE_FATAL; }
int archive_read_support_filter_uu() { return ARCHIVE_FATAL; }
int archive_read_support_filter_xz() { return ARCHIVE_FATAL; }
int archive_read_support_filter_zstd() { return ARCHIVE_FATAL; }
int archive_read_support_format_7zip() { return ARCHIVE_FATAL; }
int archive_read_support_format_all() { return ARCHIVE_FATAL; }
int archive_read_support_format_ar() { return ARCHIVE_FATAL; }
int archive_read_support_format_by_code() { return ARCHIVE_FATAL; }
int archive_read_support_format_cab() { return ARCHIVE_FATAL; }
int archive_read_support_format_cpio() { return ARCHIVE_FATAL; }
int archive_read_support_format_empty() { return ARCHIVE_FATAL; }
int archive_read_support_format_gnutar() { return ARCHIVE_FATAL; }
int archive_read_support_format_iso9660() { return ARCHIVE_FATAL; }
int archive_read_support_format_lha() { return ARCHIVE_FATAL; }
int archive_read_support_format_mtree() { return ARCHIVE_FATAL; }
int archive_read_support_format_rar() { return ARCHIVE_FATAL; }
int archive_read_support_format_rar5() { return ARCHIVE_FATAL; }
int archive_read_support_format_raw() { return ARCHIVE_FATAL; }
int archive_read_support_format_tar() { return ARCHIVE_FATAL; }
int archive_read_support_format_warc() { return ARCHIVE_FATAL; }
int archive_read_support_format_xar() { return ARCHIVE_FATAL; }
int archive_read_support_format_zip() { return ARCHIVE_FATAL; }
int archive_read_support_format_zip_streamable() { return ARCHIVE_FATAL; }
int archive_read_support_format_zip_seekable() { return ARCHIVE_FATAL; }
int archive_read_set_format() { return ARCHIVE_FATAL; }
int archive_read_append_filter() { return ARCHIVE_FATAL; }
int archive_read_append_filter_program() { return ARCHIVE_FATAL; }
int archive_read_append_filter_program_signature() { return ARCHIVE_FATAL; }
int archive_read_set_open_callback() { return ARCHIVE_FATAL; }
int archive_read_set_read_callback() { return ARCHIVE_FATAL; }
int archive_read_set_seek_callback() { return ARCHIVE_FATAL; }
int archive_read_set_skip_callback() { return ARCHIVE_FATAL; }
int archive_read_set_close_callback() { return ARCHIVE_FATAL; }
int archive_read_set_switch_callback() { return ARCHIVE_FATAL; }
int archive_read_set_callback_data() { return ARCHIVE_FATAL; }
int archive_read_set_callback_data2() { return ARCHIVE_FATAL; }
int archive_read_add_callback_data() { return ARCHIVE_FATAL; }
int archive_read_append_callback_data() { return ARCHIVE_FATAL; }
int archive_read_prepend_callback_data() { return ARCHIVE_FATAL; }
int archive_read_open1() { return ARCHIVE_FATAL; }
int archive_read_open() { return ARCHIVE_FATAL; }
int archive_read_open2() { return ARCHIVE_FATAL; }
int archive_read_open_filename() { return ARCHIVE_FATAL; }
int archive_read_open_filenames() { return ARCHIVE_FATAL; }
int archive_read_open_filename_w() { return ARCHIVE_FATAL; }
int archive_read_open_file() { return ARCHIVE_FATAL; }
int archive_read_open_memory() { return ARCHIVE_FATAL; }
int archive_read_open_memory2() { return ARCHIVE_FATAL; }
int archive_read_open_fd() { return ARCHIVE_FATAL; }
int archive_read_open_FILE() { return ARCHIVE_FATAL; }
int archive_read_next_header() { return ARCHIVE_FATAL; }
int archive_read_next_header2() { return ARCHIVE_FATAL; }
int archive_read_header_position() { return ARCHIVE_FATAL; }
int archive_read_has_encrypted_entries() { return ARCHIVE_FATAL; }
int archive_read_format_capabilities() { return ARCHIVE_FATAL; }
long long archive_read_data() { return 0; }
int archive_seek_data() { return ARCHIVE_FATAL; }
int archive_read_data_block() { return ARCHIVE_FATAL; }
int archive_read_data_skip() { return ARCHIVE_FATAL; }
int archive_read_data_into_fd() { return ARCHIVE_FATAL; }
int archive_read_set_format_option() { return ARCHIVE_FATAL; }
int archive_read_set_filter_option() { return ARCHIVE_FATAL; }
int archive_read_set_option() { return ARCHIVE_FATAL; }
int archive_read_set_options() { return ARCHIVE_FATAL; }
int archive_read_add_passphrase() { return ARCHIVE_FATAL; }
int archive_read_set_passphrase_callback() { return ARCHIVE_FATAL; }
int archive_read_extract() { return ARCHIVE_FATAL; }
int archive_read_extract2() { return ARCHIVE_FATAL; }
void archive_read_extract_set_skip_file() {  }
int archive_read_close() { return ARCHIVE_FATAL; }
int archive_read_free() { return ARCHIVE_FATAL; }
int archive_read_finish() { return ARCHIVE_FATAL; }
void* archive_write_new() { return NULL; }
int archive_write_set_bytes_per_block() { return ARCHIVE_FATAL; }
int archive_write_get_bytes_per_block() { return ARCHIVE_FATAL; }
int archive_write_set_bytes_in_last_block() { return ARCHIVE_FATAL; }
int archive_write_get_bytes_in_last_block() { return ARCHIVE_FATAL; }
int archive_write_set_skip_file() { return ARCHIVE_FATAL; }
int archive_write_set_compression_bzip2() { return ARCHIVE_FATAL; }
int archive_write_set_compression_compress() { return ARCHIVE_FATAL; }
int archive_write_set_compression_gzip() { return ARCHIVE_FATAL; }
int archive_write_set_compression_lzip() { return ARCHIVE_FATAL; }
int archive_write_set_compression_lzma() { return ARCHIVE_FATAL; }
int archive_write_set_compression_none() { return ARCHIVE_FATAL; }
int archive_write_set_compression_program() { return ARCHIVE_FATAL; }
int archive_write_set_compression_xz() { return ARCHIVE_FATAL; }
int archive_write_add_filter() { return ARCHIVE_FATAL; }
int archive_write_add_filter_by_name() { return ARCHIVE_FATAL; }
int archive_write_add_filter_b64encode() { return ARCHIVE_FATAL; }
int archive_write_add_filter_bzip2() { return ARCHIVE_FATAL; }
int archive_write_add_filter_compress() { return ARCHIVE_FATAL; }
int archive_write_add_filter_grzip() { return ARCHIVE_FATAL; }
int archive_write_add_filter_gzip() { return ARCHIVE_FATAL; }
int archive_write_add_filter_lrzip() { return ARCHIVE_FATAL; }
int archive_write_add_filter_lz4() { return ARCHIVE_FATAL; }
int archive_write_add_filter_lzip() { return ARCHIVE_FATAL; }
int archive_write_add_filter_lzma() { return ARCHIVE_FATAL; }
int archive_write_add_filter_lzop() { return ARCHIVE_FATAL; }
int archive_write_add_filter_none() { return ARCHIVE_FATAL; }
int archive_write_add_filter_program() { return ARCHIVE_FATAL; }
int archive_write_add_filter_uuencode() { return ARCHIVE_FATAL; }
int archive_write_add_filter_xz() { return ARCHIVE_FATAL; }
int archive_write_add_filter_zstd() { return ARCHIVE_FATAL; }
int archive_write_set_format() { return ARCHIVE_FATAL; }
int archive_write_set_format_by_name() { return ARCHIVE_FATAL; }
int archive_write_set_format_7zip() { return ARCHIVE_FATAL; }
int archive_write_set_format_ar_bsd() { return ARCHIVE_FATAL; }
int archive_write_set_format_ar_svr4() { return ARCHIVE_FATAL; }
int archive_write_set_format_cpio() { return ARCHIVE_FATAL; }
int archive_write_set_format_cpio_bin() { return ARCHIVE_FATAL; }
int archive_write_set_format_cpio_newc() { return 0; }
int archive_write_set_format_cpio_odc() { return ARCHIVE_FATAL; }
int archive_write_set_format_cpio_pwb() { return ARCHIVE_FATAL; }
int archive_write_set_format_gnutar() { return ARCHIVE_FATAL; }
int archive_write_set_format_iso9660() { return ARCHIVE_FATAL; }
int archive_write_set_format_mtree() { return ARCHIVE_FATAL; }
int archive_write_set_format_mtree_classic() { return ARCHIVE_FATAL; }
int archive_write_set_format_pax() { return ARCHIVE_FATAL; }
int archive_write_set_format_pax_restricted() { return ARCHIVE_FATAL; }
int archive_write_set_format_raw() { return ARCHIVE_FATAL; }
int archive_write_set_format_shar() { return ARCHIVE_FATAL; }
int archive_write_set_format_shar_dump() { return ARCHIVE_FATAL; }
int archive_write_set_format_ustar() { return ARCHIVE_FATAL; }
int archive_write_set_format_v7tar() { return ARCHIVE_FATAL; }
int archive_write_set_format_warc() { return ARCHIVE_FATAL; }
int archive_write_set_format_xar() { return ARCHIVE_FATAL; }
int archive_write_set_format_zip() { return ARCHIVE_FATAL; }
int archive_write_set_format_filter_by_ext() { return ARCHIVE_FATAL; }
int archive_write_set_format_filter_by_ext_def() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_deflate() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_store() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_lzma() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_xz() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_bzip2() { return ARCHIVE_FATAL; }
int archive_write_zip_set_compression_zstd() { return ARCHIVE_FATAL; }
int archive_write_open() { return ARCHIVE_FATAL; }
int archive_write_open2() { return ARCHIVE_FATAL; }
int archive_write_open_fd() { return ARCHIVE_FATAL; }
int archive_write_open_filename() { return ARCHIVE_FATAL; }
int archive_write_open_filename_w() { return ARCHIVE_FATAL; }
int archive_write_open_file() { return ARCHIVE_FATAL; }
int archive_write_open_FILE() { return ARCHIVE_FATAL; }
int archive_write_open_memory() { return ARCHIVE_FATAL; }
int archive_write_header() { return ARCHIVE_FATAL; }
long long archive_write_data() { return 0; }
long long archive_write_data_block() { return 0; }
int archive_write_finish_entry() { return ARCHIVE_FATAL; }
int archive_write_close() { return ARCHIVE_FATAL; }
int archive_write_fail() { return ARCHIVE_FATAL; }
int archive_write_free() { return ARCHIVE_FATAL; }
int archive_write_finish() { return ARCHIVE_FATAL; }
int archive_write_set_format_option() { return ARCHIVE_FATAL; }
int archive_write_set_filter_option() { return ARCHIVE_FATAL; }
int archive_write_set_option() { return ARCHIVE_FATAL; }
int archive_write_set_options() { return ARCHIVE_FATAL; }
int archive_write_set_passphrase() { return ARCHIVE_FATAL; }
int archive_write_set_passphrase_callback() { return ARCHIVE_FATAL; }
void* archive_write_disk_new() { return NULL; }
int archive_write_disk_set_skip_file() { return ARCHIVE_FATAL; }
int archive_write_disk_set_options() { return ARCHIVE_FATAL; }
int archive_write_disk_set_standard_lookup() { return ARCHIVE_FATAL; }
int archive_write_disk_set_group_lookup() { return ARCHIVE_FATAL; }
int archive_write_disk_set_user_lookup() { return ARCHIVE_FATAL; }
int archive_write_disk_gid() { return ARCHIVE_FATAL; }
int archive_write_disk_uid() { return ARCHIVE_FATAL; }
void* archive_read_disk_new() { return NULL; }
int archive_read_disk_set_symlink_logical() { return ARCHIVE_FATAL; }
int archive_read_disk_set_symlink_physical() { return ARCHIVE_FATAL; }
int archive_read_disk_set_symlink_hybrid() { return ARCHIVE_FATAL; }
int archive_read_disk_entry_from_file() { return ARCHIVE_FATAL; }
void* archive_read_disk_gname() { return NULL; }
void* archive_read_disk_uname() { return NULL; }
int archive_read_disk_set_standard_lookup() { return ARCHIVE_FATAL; }
void* archive_read_disk_set_gname_lookup() { return NULL; }
void* archive_read_disk_set_uname_lookup() { return NULL; }
int archive_read_disk_open() { return ARCHIVE_FATAL; }
int archive_read_disk_open_w() { return ARCHIVE_FATAL; }
int archive_read_disk_descend() { return ARCHIVE_FATAL; }
int archive_read_disk_can_descend() { return ARCHIVE_FATAL; }
int archive_read_disk_current_filesystem() { return ARCHIVE_FATAL; }
int archive_read_disk_current_filesystem_is_synthetic() { return ARCHIVE_FATAL; }
int archive_read_disk_current_filesystem_is_remote() { return ARCHIVE_FATAL; }
int archive_read_disk_set_atime_restored() { return ARCHIVE_FATAL; }
int archive_read_disk_set_behavior() { return ARCHIVE_FATAL; }
int archive_read_disk_set_metadata_filter_callback() { return ARCHIVE_FATAL; }
int archive_free() { return ARCHIVE_FATAL; }
int archive_filter_count() { return ARCHIVE_FATAL; }
int archive_filter_bytes() { return ARCHIVE_FATAL; }
int archive_filter_code() { return ARCHIVE_FATAL; }
void* archive_filter_name() { return NULL; }
int archive_position_compressed() { return ARCHIVE_FATAL; }
int archive_position_uncompressed() { return ARCHIVE_FATAL; }
void* archive_compression_name() { return NULL; }
int archive_compression() { return ARCHIVE_FATAL; }
int archive_parse_date() { return ARCHIVE_FATAL; }
int archive_errno() { return ARCHIVE_FATAL; }
void* archive_error_string() { return NULL; }
void* archive_format_name() { return NULL; }
int archive_format() { return ARCHIVE_FATAL; }
void archive_clear_error() {  }
void archive_set_error() {  }
void archive_copy_error() {  }
int archive_file_count() { return ARCHIVE_FATAL; }
void* archive_match_new() { return NULL; }
int archive_match_free() { return ARCHIVE_FATAL; }
int archive_match_excluded() { return ARCHIVE_FATAL; }
int archive_match_path_excluded() { return ARCHIVE_FATAL; }
int archive_match_set_inclusion_recursion() { return ARCHIVE_FATAL; }
int archive_match_exclude_pattern() { return ARCHIVE_FATAL; }
int archive_match_exclude_pattern_w() { return ARCHIVE_FATAL; }
int archive_match_exclude_pattern_from_file() { return ARCHIVE_FATAL; }
int archive_match_exclude_pattern_from_file_w() { return ARCHIVE_FATAL; }
int archive_match_include_pattern() { return ARCHIVE_FATAL; }
int archive_match_include_pattern_w() { return ARCHIVE_FATAL; }
int archive_match_include_pattern_from_file() { return ARCHIVE_FATAL; }
int archive_match_include_pattern_from_file_w() { return ARCHIVE_FATAL; }
int archive_match_path_unmatched_inclusions() { return ARCHIVE_FATAL; }
int archive_match_path_unmatched_inclusions_next() { return ARCHIVE_FATAL; }
int archive_match_path_unmatched_inclusions_next_w() { return ARCHIVE_FATAL; }
int archive_match_time_excluded() { return ARCHIVE_FATAL; }
int archive_match_include_time() { return ARCHIVE_FATAL; }
int archive_match_include_date() { return ARCHIVE_FATAL; }
int archive_match_include_date_w() { return ARCHIVE_FATAL; }
int archive_match_include_file_time() { return ARCHIVE_FATAL; }
int archive_match_include_file_time_w() { return ARCHIVE_FATAL; }
int archive_match_exclude_entry() { return ARCHIVE_FATAL; }
int archive_match_owner_excluded() { return ARCHIVE_FATAL; }
int archive_match_include_uid() { return ARCHIVE_FATAL; }
int archive_match_include_gid() { return ARCHIVE_FATAL; }
int archive_match_include_uname() { return ARCHIVE_FATAL; }
int archive_match_include_uname_w() { return ARCHIVE_FATAL; }
int archive_match_include_gname() { return ARCHIVE_FATAL; }
int archive_match_include_gname_w() { return ARCHIVE_FATAL; }
int archive_utility_string_sort() { return ARCHIVE_FATAL; }
void* archive_entry_clear() { return NULL; }
void* archive_entry_clone() { return NULL; }
void archive_entry_free() {  }
void* archive_entry_new() { return NULL; }
void* archive_entry_new2() { return NULL; }
int archive_entry_atime() { return ARCHIVE_FATAL; }
int archive_entry_atime_nsec() { return ARCHIVE_FATAL; }
int archive_entry_atime_is_set() { return ARCHIVE_FATAL; }
int archive_entry_birthtime() { return ARCHIVE_FATAL; }
int archive_entry_birthtime_nsec() { return ARCHIVE_FATAL; }
int archive_entry_birthtime_is_set() { return ARCHIVE_FATAL; }
int archive_entry_ctime() { return ARCHIVE_FATAL; }
int archive_entry_ctime_nsec() { return ARCHIVE_FATAL; }
int archive_entry_ctime_is_set() { return ARCHIVE_FATAL; }
int archive_entry_dev() { return ARCHIVE_FATAL; }
int archive_entry_dev_is_set() { return ARCHIVE_FATAL; }
int archive_entry_devmajor() { return ARCHIVE_FATAL; }
int archive_entry_devminor() { return ARCHIVE_FATAL; }
int archive_entry_filetype() { return ARCHIVE_FATAL; }
int archive_entry_filetype_is_set() { return ARCHIVE_FATAL; }
void archive_entry_fflags() {  }
void* archive_entry_fflags_text() { return NULL; }
int archive_entry_gid() { return ARCHIVE_FATAL; }
int archive_entry_gid_is_set() { return ARCHIVE_FATAL; }
void* archive_entry_gname() { return NULL; }
void* archive_entry_gname_utf8() { return NULL; }
void* archive_entry_gname_w() { return NULL; }
void archive_entry_set_link_to_hardlink() {  }
void* archive_entry_hardlink() { return NULL; }
void* archive_entry_hardlink_utf8() { return NULL; }
void* archive_entry_hardlink_w() { return NULL; }
int archive_entry_hardlink_is_set() { return ARCHIVE_FATAL; }
int archive_entry_ino() { return ARCHIVE_FATAL; }
int archive_entry_ino64() { return ARCHIVE_FATAL; }
int archive_entry_ino_is_set() { return ARCHIVE_FATAL; }
int archive_entry_mode() { return ARCHIVE_FATAL; }
int archive_entry_mtime() { return ARCHIVE_FATAL; }
int archive_entry_mtime_nsec() { return ARCHIVE_FATAL; }
int archive_entry_mtime_is_set() { return ARCHIVE_FATAL; }
unsigned int archive_entry_nlink() { return 0; }
void* archive_entry_pathname() { return NULL; }
void* archive_entry_pathname_utf8() { return NULL; }
void* archive_entry_pathname_w() { return NULL; }
int archive_entry_perm() { return ARCHIVE_FATAL; }
int archive_entry_perm_is_set() { return ARCHIVE_FATAL; }
int archive_entry_rdev_is_set() { return ARCHIVE_FATAL; }
int archive_entry_rdev() { return ARCHIVE_FATAL; }
int archive_entry_rdevmajor() { return ARCHIVE_FATAL; }
int archive_entry_rdevminor() { return ARCHIVE_FATAL; }
void* archive_entry_sourcepath() { return NULL; }
void* archive_entry_sourcepath_w() { return NULL; }
int archive_entry_size() { return ARCHIVE_FATAL; }
int archive_entry_size_is_set() { return ARCHIVE_FATAL; }
void* archive_entry_strmode() { return NULL; }
void archive_entry_set_link_to_symlink() {  }
void* archive_entry_symlink() { return NULL; }
void* archive_entry_symlink_utf8() { return NULL; }
int archive_entry_symlink_type() { return ARCHIVE_FATAL; }
void* archive_entry_symlink_w() { return NULL; }
int archive_entry_uid() { return ARCHIVE_FATAL; }
int archive_entry_uid_is_set() { return ARCHIVE_FATAL; }
void* archive_entry_uname() { return NULL; }
void* archive_entry_uname_utf8() { return NULL; }
void* archive_entry_uname_w() { return NULL; }
int archive_entry_is_data_encrypted() { return ARCHIVE_FATAL; }
int archive_entry_is_metadata_encrypted() { return ARCHIVE_FATAL; }
int archive_entry_is_encrypted() { return ARCHIVE_FATAL; }
void archive_entry_set_atime() {  }
void archive_entry_unset_atime() {  }
void archive_entry_set_birthtime() {  }
void archive_entry_unset_birthtime() {  }
void archive_entry_set_ctime() {  }
void archive_entry_unset_ctime() {  }
void archive_entry_set_dev() {  }
void archive_entry_set_devmajor() {  }
void archive_entry_set_devminor() {  }
void archive_entry_set_filetype() {  }
void archive_entry_set_fflags() {  }
void* archive_entry_copy_fflags_text() { return NULL; }
void* archive_entry_copy_fflags_text_len() { return NULL; }
void* archive_entry_copy_fflags_text_w() { return NULL; }
void archive_entry_set_gid() {  }
void archive_entry_set_gname() {  }
void archive_entry_set_gname_utf8() {  }
void archive_entry_copy_gname() {  }
void archive_entry_copy_gname_w() {  }
int archive_entry_update_gname_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_hardlink() {  }
void archive_entry_set_hardlink_utf8() {  }
void archive_entry_copy_hardlink() {  }
void archive_entry_copy_hardlink_w() {  }
int archive_entry_update_hardlink_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_ino() {  }
void archive_entry_set_ino64() {  }
void archive_entry_set_link() {  }
void archive_entry_set_link_utf8() {  }
void archive_entry_copy_link() {  }
void archive_entry_copy_link_w() {  }
int archive_entry_update_link_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_mode() {  }
void archive_entry_set_mtime() {  }
void archive_entry_unset_mtime() {  }
void archive_entry_set_nlink() {  }
void archive_entry_set_pathname() {  }
void archive_entry_set_pathname_utf8() {  }
void archive_entry_copy_pathname() {  }
void archive_entry_copy_pathname_w() {  }
int archive_entry_update_pathname_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_perm() {  }
void archive_entry_set_rdev() {  }
void archive_entry_set_rdevmajor() {  }
void archive_entry_set_rdevminor() {  }
void archive_entry_set_size() {  }
void archive_entry_unset_size() {  }
void archive_entry_copy_sourcepath() {  }
void archive_entry_copy_sourcepath_w() {  }
void archive_entry_set_symlink() {  }
void archive_entry_set_symlink_type() {  }
void archive_entry_set_symlink_utf8() {  }
void archive_entry_copy_symlink() {  }
void archive_entry_copy_symlink_w() {  }
int archive_entry_update_symlink_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_uid() {  }
void archive_entry_set_uname() {  }
void archive_entry_set_uname_utf8() {  }
void archive_entry_copy_uname() {  }
void archive_entry_copy_uname_w() {  }
int archive_entry_update_uname_utf8() { return ARCHIVE_FATAL; }
void archive_entry_set_is_data_encrypted() {  }
void archive_entry_set_is_metadata_encrypted() {  }
void* archive_entry_stat() { return NULL; }
void archive_entry_copy_stat() {  }
void* archive_entry_mac_metadata() { return NULL; }
void archive_entry_copy_mac_metadata() {  }
void* archive_entry_digest() { return NULL; }
int archive_entry_set_digest() { return ARCHIVE_FATAL; }
void archive_entry_acl_clear() {  }
int archive_entry_acl_add_entry() { return ARCHIVE_FATAL; }
int archive_entry_acl_add_entry_w() { return ARCHIVE_FATAL; }
int archive_entry_acl_reset() { return ARCHIVE_FATAL; }
int archive_entry_acl_next() { return ARCHIVE_FATAL; }
int archive_entry_acl_to_text_w() { return ARCHIVE_FATAL; }
const char* archive_entry_acl_to_text() { return NULL; }
int archive_entry_acl_from_text_w() { return ARCHIVE_FATAL; }
int archive_entry_acl_from_text() { return ARCHIVE_FATAL; }
void* archive_entry_acl_text_w() { return NULL; }
void* archive_entry_acl_text() { return NULL; }
int archive_entry_acl_types() { return ARCHIVE_FATAL; }
int archive_entry_acl_count() { return ARCHIVE_FATAL; }
void* archive_entry_acl() { return NULL; }
void archive_entry_xattr_clear() {  }
void archive_entry_xattr_add_entry() {  }
int archive_entry_xattr_count() { return ARCHIVE_FATAL; }
int archive_entry_xattr_reset() { return ARCHIVE_FATAL; }
int archive_entry_xattr_next() { return ARCHIVE_FATAL; }
void archive_entry_sparse_clear() {  }
void archive_entry_sparse_add_entry() {  }
int archive_entry_sparse_count() { return ARCHIVE_FATAL; }
int archive_entry_sparse_reset() { return ARCHIVE_FATAL; }
int archive_entry_sparse_next() { return ARCHIVE_FATAL; }
void* archive_entry_linkresolver_new() { return NULL; }
void archive_entry_linkresolver_set_strategy() {  }
void archive_entry_linkresolver_free() {  }
void archive_entry_linkify() {  }
void* archive_entry_partial_links() { return NULL; }
