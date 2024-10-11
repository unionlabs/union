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

package transfermanager_test

import (
	"context"
	"log"
	"os"

	"cloud.google.com/go/storage"
	"cloud.google.com/go/storage/transfermanager"
)

func ExampleDownloader_synchronous() {
	ctx := context.Background()
	// Pass in any client opts or set retry policy here.
	client, err := storage.NewClient(ctx) // can also use NewGRPCClient
	if err != nil {
		// handle error
	}

	// Create Downloader with desired options, including number of workers,
	// part size, per operation timeout, etc.
	d, err := transfermanager.NewDownloader(client, transfermanager.WithWorkers(16))
	if err != nil {
		// handle error
	}

	// Create local file writer for output.
	f, err := os.Create("/path/to/localfile")
	if err != nil {
		// handle error
	}

	// Create download input
	in := &transfermanager.DownloadObjectInput{
		Bucket:      "mybucket",
		Object:      "myblob",
		Destination: f,
		// Optionally specify params to apply to download.
		EncryptionKey: []byte("mykey"),
	}

	// Add to Downloader.
	if err := d.DownloadObject(ctx, in); err != nil {
		// handle error
	}

	// Repeat if desired.

	// Wait for all downloads to complete.
	results, err := d.WaitAndClose()
	if err != nil {
		// handle error
	}

	// Iterate through completed downloads and process results.
	for _, out := range results {
		if out.Err != nil {
			log.Printf("download of %v failed with error %v", out.Object, out.Err)
		} else {
			log.Printf("download of %v succeeded", out.Object)
		}
	}
}

func ExampleDownloader_asynchronous() {
	ctx := context.Background()
	// Pass in any client opts or set retry policy here.
	client, err := storage.NewClient(ctx) // can also use NewGRPCClient
	if err != nil {
		// handle error
	}

	// Create Downloader with callbacks plus any desired options, including
	// number of workers, part size, per operation timeout, etc.
	d, err := transfermanager.NewDownloader(client, transfermanager.WithCallbacks())
	if err != nil {
		// handle error
	}
	defer func() {
		if _, err := d.WaitAndClose(); err != nil {
			// one or more of the downloads failed
		}
	}()

	// Create local file writer for output.
	f, err := os.Create("/path/to/localfile")
	if err != nil {
		// handle error
	}

	// Create callback function
	callback := func(out *transfermanager.DownloadOutput) {
		if out.Err != nil {
			log.Printf("download of %v failed with error %v", out.Object, out.Err)
		} else {
			log.Printf("download of %v succeeded", out.Object)
		}
	}

	// Create download input
	in := &transfermanager.DownloadObjectInput{
		Bucket:      "mybucket",
		Object:      "myblob",
		Destination: f,
		// Optionally specify params to apply to download.
		EncryptionKey: []byte("mykey"),
		// Specify the callback
		Callback: callback,
	}

	// Add to Downloader.
	if err := d.DownloadObject(ctx, in); err != nil {
		// handle error
	}

	// Repeat if desired.
}
