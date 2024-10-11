package gzip

import (
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"io/ioutil"
	"strconv"

	"github.com/aws/aws-sdk-go/aws/request"
)

// NewGzipRequestHandler provides a named request handler that compresses the
// request payload.  Add this to enable GZIP compression for a client.
//
// Known to work with Amazon CloudWatch's PutMetricData operation.
// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricData.html
func NewGzipRequestHandler() request.NamedHandler {
	return request.NamedHandler{
		Name: "GzipRequestHandler",
		Fn:   gzipRequestHandler,
	}
}

func gzipRequestHandler(req *request.Request) {
	compressedBytes, err := compress(req.Body)
	if err != nil {
		req.Error = fmt.Errorf("failed to compress request payload, %v", err)
		return
	}

	req.HTTPRequest.Header.Set("Content-Encoding", "gzip")
	req.HTTPRequest.Header.Set("Content-Length", strconv.Itoa(len(compressedBytes)))

	req.SetBufferBody(compressedBytes)
}

func compress(input io.Reader) ([]byte, error) {
	var b bytes.Buffer
	w, err := gzip.NewWriterLevel(&b, gzip.BestCompression)
	if err != nil {
		return nil, fmt.Errorf("failed to create gzip writer, %v", err)
	}

	inBytes, err := ioutil.ReadAll(input)
	if err != nil {
		return nil, fmt.Errorf("failed read payload to compress, %v", err)
	}

	if _, err = w.Write(inBytes); err != nil {
		return nil, fmt.Errorf("failed to write payload to be compressed, %v", err)
	}
	if err = w.Close(); err != nil {
		return nil, fmt.Errorf("failed to flush payload being compressed, %v", err)
	}

	return b.Bytes(), nil
}
