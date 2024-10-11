#!/usr/bin/env python
"""
This script rewites the zstd source files to enclose the source code
inside a #ifndef USE_EXTERNAL_ZSTD.

The goal of that is to avoid compiling vendored zstd source files
when we compile the library against an externally provided libzstd.
"""
import sys
import glob

FLAG="USE_EXTERNAL_ZSTD"

HEADER=f"""#ifndef {FLAG}
"""

ZSTD_H_FOOTER=f"""
#else /* {FLAG} */
#include_next <zstd.h>
#endif /* {FLAG} */
"""

FOOTER=f"""
#endif /* {FLAG} */
"""

def patch_file_content(filename, content):
    new_content = ""
    if not content.startswith(HEADER):
        new_content += HEADER
    new_content+=content
    footer = ZSTD_H_FOOTER if filename == "zstd.h" else FOOTER
    if not content.endswith(footer):
        new_content += footer
    return new_content

def insert_ifdefs(file):
    with open(file, "r") as fd:
        content=fd.read()
    with open(file, "w") as fd:
        fd.write(patch_file_content(file, content))

if __name__ == "__main__":
    for file in glob.glob("*.c") + glob.glob("*.h"):
        insert_ifdefs(file)
