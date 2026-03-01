#!/usr/bin/env python3
"""Generate stub C library for libarchive cross-compilation."""
import re
import sys

bindings_path = "src-tauri/target/release/build/libarchive2-sys-07ea7437653e015a/out/bindings.rs"
output_path = "patches/libarchive2-sys/src/stub_archive.c"

with open(bindings_path) as f:
    content = f.read()

# Find all extern function declarations
pattern = r'pub fn (archive_\w+)\s*\(([^)]*)\)\s*(?:->\s*([^;]+))?;'
matches = re.findall(pattern, content)

lines = [
    "/* Auto-generated stub library for cross-compilation */",
    "/* Archive operations will return errors at runtime */",
    "",
    "#include <stddef.h>",
    "#include <stdint.h>",
    "",
    "#define ARCHIVE_FATAL -30",
    "#define ARCHIVE_OK 0",
    "",
    "struct archive;",
    "struct archive_entry;",
    "struct archive_entry_linkresolver;",
    "struct archive_match;",
    "struct archive_read_disk;",
    "",
]

for name, args, ret in matches:
    ret = ret.strip() if ret else ""

    if not ret or ret == "()":
        c_ret = "void"
    elif "*mut archive" in ret or "*mut archive_entry" in ret or "*const" in ret:
        c_ret = "void*"
    elif ret in ("i64", "::std::os::raw::c_longlong"):
        c_ret = "long long"
    elif ret in ("i32", "::std::os::raw::c_int"):
        c_ret = "int"
    elif ret in ("u32", "::std::os::raw::c_uint"):
        c_ret = "unsigned int"
    elif ret == "u64":
        c_ret = "unsigned long long"
    elif "c_char" in ret:
        c_ret = "const char*"
    elif "isize" in ret or "la_ssize_t" in ret:
        c_ret = "long long"
    elif "usize" in ret:
        c_ret = "unsigned long long"
    else:
        c_ret = "int"

    if c_ret == "void":
        ret_val = ""
    elif c_ret in ("void*",) or "char*" in c_ret:
        ret_val = "return NULL;"
    elif c_ret == "int":
        if "new" in name or "version" in name:
            ret_val = "return 0;"
        else:
            ret_val = "return ARCHIVE_FATAL;"
    else:
        ret_val = "return 0;"

    lines.append(f"{c_ret} {name}() {{ {ret_val} }}")

with open(output_path, "w") as f:
    f.write("\n".join(lines) + "\n")

print(f"{len(matches)} functions stubbed -> {output_path}")
