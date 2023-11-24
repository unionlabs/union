# atomicfile

[![GoDoc](https://img.shields.io/static/v1?label=godoc&message=reference&color=blue)](https://pkg.go.dev/github.com/creachadair/atomicfile)

Package atomicfile implements all-or-nothing file replacement by staging output
to a temporary file adjacent to the specified target file, and renaming over
the target when the temporary is closed.
