// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package transfermanager

import (
	"context"
	"errors"
	"fmt"
	"io"
	"sync"
	"time"

	"cloud.google.com/go/storage"
	"github.com/googleapis/gax-go/v2/callctx"
)

// Downloader manages a set of parallelized downloads.
type Downloader struct {
	client    *storage.Client
	config    *transferManagerConfig
	inputs    []DownloadObjectInput
	results   []DownloadOutput
	errors    []error
	inputsMu  *sync.Mutex
	resultsMu *sync.Mutex
	errorsMu  *sync.Mutex
	work      chan *DownloadObjectInput // Piece of work to be executed.
	done      chan bool                 // Indicates to finish up work; expecting no more inputs.
	workers   *sync.WaitGroup           // Keeps track of the workers that are currently running.
}

// DownloadObject queues the download of a single object. This will initiate the
// download but is non-blocking; call Downloader.Results or use the callback to
// process the result. DownloadObject is thread-safe and can be called
// simultaneously from different goroutines.
// The download may not start immediately if all workers are busy, so a deadline
// set on the ctx may time out before the download even starts. To set a timeout
// that starts with the download, use the [WithPerOpTimeout()] option.
func (d *Downloader) DownloadObject(ctx context.Context, input *DownloadObjectInput) error {
	if d.config.asynchronous && input.Callback == nil {
		return errors.New("transfermanager: input.Callback must not be nil when the WithCallbacks option is set")
	}
	if !d.config.asynchronous && input.Callback != nil {
		return errors.New("transfermanager: input.Callback must be nil unless the WithCallbacks option is set")
	}

	select {
	case <-d.done:
		return errors.New("transfermanager: WaitAndClose called before DownloadObject")
	default:
	}

	input.ctx = ctx
	d.addInput(input)
	return nil
}

// WaitAndClose waits for all outstanding downloads to complete and closes the
// Downloader. Adding new downloads after this has been called will cause an error.
//
// WaitAndClose returns all the results of the downloads and an error wrapping
// all errors that were encountered by the Downloader when downloading objects.
// These errors are also returned in the respective DownloadOutput for the
// failing download. The results are not guaranteed to be in any order.
// Results will be empty if using the [WithCallbacks] option.
func (d *Downloader) WaitAndClose() ([]DownloadOutput, error) {
	errMsg := "transfermanager: at least one error encountered downloading objects:"
	select {
	case <-d.done: // this allows users to call WaitAndClose various times
		var err error
		if len(d.errors) > 0 {
			err = fmt.Errorf("%s\n%w", errMsg, errors.Join(d.errors...))
		}
		return d.results, err
	default:
		d.done <- true
		d.workers.Wait()
		close(d.done)

		if len(d.errors) > 0 {
			return d.results, fmt.Errorf("%s\n%w", errMsg, errors.Join(d.errors...))
		}
		return d.results, nil
	}
}

// sendInputsToWorkChan listens continuously to the inputs slice until d.done.
// It will send all items in inputs to the d.work chan.
// Once it receives from d.done, it drains the remaining items in the inputs
// (sending them to d.work) and then closes the d.work chan.
func (d *Downloader) sendInputsToWorkChan() {
	for {
		select {
		case <-d.done:
			d.drainInput()
			close(d.work)
			return
		default:
			d.drainInput()
		}
	}
}

// drainInput consumes everything in the inputs slice and sends it to the work chan.
// It will block if there are not enough workers to consume every input, until all
// inputs are received on the work chan(ie. they're dispatched to an available worker).
func (d *Downloader) drainInput() {
	for {
		d.inputsMu.Lock()
		if len(d.inputs) < 1 {
			d.inputsMu.Unlock()
			return
		}
		input := d.inputs[0]
		d.inputs = d.inputs[1:]
		d.inputsMu.Unlock()
		d.work <- &input
	}
}

func (d *Downloader) addInput(input *DownloadObjectInput) {
	d.inputsMu.Lock()
	d.inputs = append(d.inputs, *input)
	d.inputsMu.Unlock()
}

func (d *Downloader) addResult(result *DownloadOutput) {
	d.resultsMu.Lock()
	d.results = append(d.results, *result)
	d.resultsMu.Unlock()
}

func (d *Downloader) error(err error) {
	d.errorsMu.Lock()
	d.errors = append(d.errors, err)
	d.errorsMu.Unlock()
}

// downloadWorker continuously processes downloads until the work channel is closed.
func (d *Downloader) downloadWorker() {
	for {
		input, ok := <-d.work
		if !ok {
			break // no more work; exit
		}

		// TODO: break down the input into smaller pieces if necessary; maybe as follows:
		// Only request partSize data to begin with. If no error and we haven't finished
		// reading the object, enqueue the remaining pieces of work and mark in the
		// out var the amount of shards to wait for.
		out := input.downloadShard(d.client, d.config.perOperationTimeout)

		// Keep track of any error that occurred.
		if out.Err != nil {
			d.error(fmt.Errorf("downloading %q from bucket %q: %w", input.Object, input.Bucket, out.Err))
		}

		// Either execute the callback, or append to results.
		if d.config.asynchronous {
			input.Callback(out)
		} else {
			d.addResult(out)
		}
	}
	d.workers.Done()
}

// NewDownloader creates a new Downloader to add operations to.
// Choice of transport, etc is configured on the client that's passed in.
// The returned Downloader can be shared across goroutines to initiate downloads.
func NewDownloader(c *storage.Client, opts ...Option) (*Downloader, error) {
	d := &Downloader{
		client:    c,
		config:    initTransferManagerConfig(opts...),
		inputs:    []DownloadObjectInput{},
		results:   []DownloadOutput{},
		errors:    []error{},
		inputsMu:  &sync.Mutex{},
		resultsMu: &sync.Mutex{},
		errorsMu:  &sync.Mutex{},
		work:      make(chan *DownloadObjectInput),
		done:      make(chan bool),
		workers:   &sync.WaitGroup{},
	}

	// Start a listener to send work through.
	go d.sendInputsToWorkChan()

	// Start workers.
	for i := 0; i < d.config.numWorkers; i++ {
		d.workers.Add(1)
		go d.downloadWorker()
	}

	return d, nil
}

// DownloadRange specifies the object range.
type DownloadRange struct {
	// Offset is the starting offset (inclusive) from with the object is read.
	// If offset is negative, the object is read abs(offset) bytes from the end,
	// and length must also be negative to indicate all remaining bytes will be read.
	Offset int64
	// Length is the number of bytes to read.
	// If length is negative or larger than the object size, the object is read
	// until the end.
	Length int64
}

// DownloadObjectInput is the input for a single object to download.
type DownloadObjectInput struct {
	// Required fields
	Bucket      string
	Object      string
	Destination io.WriterAt

	// Optional fields
	Generation    *int64
	Conditions    *storage.Conditions
	EncryptionKey []byte
	Range         *DownloadRange // if specified, reads only a range

	// Callback will be run once the object is finished downloading. It must be
	// set if and only if the [WithCallbacks] option is set; otherwise, it must
	// not be set.
	Callback func(*DownloadOutput)

	ctx context.Context
}

// downloadShard will read a specific object into in.Destination.
// If timeout is less than 0, no timeout is set.
// TODO: download a single shard instead of the entire object.
func (in *DownloadObjectInput) downloadShard(client *storage.Client, timeout time.Duration) (out *DownloadOutput) {
	out = &DownloadOutput{Bucket: in.Bucket, Object: in.Object, Range: in.Range}

	// Set timeout.
	ctx := in.ctx
	if timeout > 0 {
		c, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()
		ctx = c
	}

	// TODO: set to downloadSharded when sharded
	ctx = setUsageMetricHeader(ctx, downloadMany)

	// Set options on the object.
	o := client.Bucket(in.Bucket).Object(in.Object)

	if in.Conditions != nil {
		o = o.If(*in.Conditions)
	}
	if in.Generation != nil {
		o = o.Generation(*in.Generation)
	}
	if len(in.EncryptionKey) > 0 {
		o = o.Key(in.EncryptionKey)
	}

	var offset, length int64 = 0, -1 // get the entire object by default

	if in.Range != nil {
		offset, length = in.Range.Offset, in.Range.Length
	}

	// Read.
	r, err := o.NewRangeReader(ctx, offset, length)
	if err != nil {
		out.Err = err
		return
	}

	// TODO: write at a specific offset.
	off := io.NewOffsetWriter(in.Destination, 0)
	_, err = io.Copy(off, r)
	if err != nil {
		out.Err = err
		r.Close()
		return
	}

	if err = r.Close(); err != nil {
		out.Err = err
		return
	}

	out.Attrs = &r.Attrs
	return
}

// DownloadOutput provides output for a single object download, including all
// errors received while downloading object parts. If the download was successful,
// Attrs will be populated.
type DownloadOutput struct {
	Bucket string
	Object string
	Range  *DownloadRange             // requested range, if it was specified
	Err    error                      // error occurring during download
	Attrs  *storage.ReaderObjectAttrs // attributes of downloaded object, if successful
}

const (
	xGoogHeaderKey  = "x-goog-api-client"
	usageMetricKey  = "gccl-gcs-cmd"
	downloadMany    = "tm.download_many"
	downloadSharded = "tm.download_sharded"
)

// Sets invocation ID headers on the context which will be propagated as
// headers in the call to the service (for both gRPC and HTTP).
func setUsageMetricHeader(ctx context.Context, method string) context.Context {
	header := fmt.Sprintf("%s/%s", usageMetricKey, method)
	return callctx.SetHeaders(ctx, xGoogHeaderKey, header)
}
