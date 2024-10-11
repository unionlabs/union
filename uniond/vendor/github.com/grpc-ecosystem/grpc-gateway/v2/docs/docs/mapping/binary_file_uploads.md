---
layout: default
title: Binary file uploads
nav_order: 2
parent: Mapping
---

# Binary file uploads

If you need to do a binary file upload, e.g. via;

```sh
curl -X POST -F "attachment=@/tmp/somefile.txt" http://localhost:9090/v1/files
```

then your request will contain the binary data directly and there is no way to model this using gRPC.

What you can do instead is to add a custom route directly on the `mux` instance.

## Custom route on a mux instance

Here we'll setup a handler (`handleBinaryFileUpload`) for `POST` requests: 

```go
// Create a mux instance
mux := runtime.NewServeMux()

// Attachment upload from http/s handled manually
mux.HandlePath("POST", "/v1/files", handleBinaryFileUpload)
```

And then in your handler you can do something like:

```go
func handleBinaryFileUpload(w http.ResponseWriter, r *http.Request, params map[string]string) {
	err := r.ParseForm()
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to parse form: %s", err.Error()), http.StatusBadRequest)
		return
	}

	f, header, err := r.FormFile("attachment")
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to get file 'attachment': %s", err.Error()), http.StatusBadRequest)
		return
	}
	defer f.Close()

	//
	// Now do something with the io.Reader in `f`, i.e. read it into a buffer or stream it to a gRPC client side stream.
	// Also `header` will contain the filename, size etc of the original file.
	//
}
```
