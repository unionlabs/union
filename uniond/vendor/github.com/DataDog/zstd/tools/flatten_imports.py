#!/usr/bin/env python
"""
This script rewites the zstd source files to flatten imports
"""

import glob

def rewrite_file(path):
	results = []
	with open(path, "r") as f: 
		for l in f.readlines():
			line_no_space = l.replace(" ", "")
			if not line_no_space.startswith('#include"..'):
				results.append(l) # Do nothing
			else:
				# Include line, rewrite it
				new_path = l.split('"')[1]
				end = l.split('"')[-1]
				new_path = new_path.split("/")[-1]
				results.append('#include "' + new_path + '"' + end)
	with open(path, "w") as f:
		for l in results:
			f.write(l)


if __name__ == "__main__":
    for file in glob.glob("*.c") + glob.glob("*.h") + glob.glob("*.S"):
        rewrite_file(file)
