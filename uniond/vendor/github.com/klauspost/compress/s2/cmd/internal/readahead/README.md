# readahead
Asynchronous read-ahead for Go readers

This package will allow you to add readhead to any reader. This means a separate goroutine will perform reads from your upstream reader, so you can request from this reader without delay.

This is helpful for splitting an input stream into concurrent processing, and also helps smooth out **bursts** of input or output.

This should be fully transparent, except that once an error has been returned from the Reader, it will not recover. A panic will be caught and returned as an error.

The readahead object also fulfills the [`io.WriterTo`](https://golang.org/pkg/io/#WriterTo) interface, which is likely to speed up `io.Copy` and other code that use the interface.

See an introduction: [An Async Read-ahead Package for Go](https://blog.klauspost.com/an-async-read-ahead-package-for-go/)

[![GoDoc][1]][2] [![Build Status][3]][4]

[1]: https://godoc.org/github.com/klauspost/readahead?status.svg
[2]: https://godoc.org/github.com/klauspost/readahead
[3]: https://travis-ci.org/klauspost/readahead.svg
[4]: https://travis-ci.org/klauspost/readahead

# usage

To get the package use `go get -u github.com/klauspost/readahead`.

Here is a simple example that does file copy. Error handling has been omitted for brevity.
```Go
input, _ := os.Open("input.txt")
output, _ := os.Create("output.txt")
defer input.Close()
defer output.Close()

// Create a read-ahead Reader with default settings
ra := readahead.NewReader(input)
defer ra.Close()

// Copy the content to our output
_, _ = io.Copy(output, ra)
```

# settings

You can finetune the read-ahead for your specific use case, and adjust the number of buffers and the size of each buffer.

The default the size of each buffer is 1MB, and there are 4 buffers. Do not make your buffers too small since there is a small overhead for passing buffers between goroutines. Other than that you are free to experiment with buffer sizes.

# contributions

On this project contributions in terms of new features is limited to:

* Features that are widely usable and
* Features that have extensive tests

This package is meant to be simple and stable, so therefore these strict requirements.

# license

This package is released under the MIT license. See the supplied LICENSE file for more info.
