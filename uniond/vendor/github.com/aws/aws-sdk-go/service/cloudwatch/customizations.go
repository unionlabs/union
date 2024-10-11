package cloudwatch

import (
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/internal/encoding/gzip"
)

// WithGzipRequest is a request.Option that adds a request handler to the Build
// stage of the operation's pipeline that will content-encoding GZIP the
// request payload before sending it to the API. This will buffer the request
// payload in memory, GZIP it, and reassign the GZIP'ed payload as the new
// request payload.
//
// GZIP may not be supported by all API operations. See API's documentation for
// the operation your using to see if GZIP request payload is supported.
// https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricData.html
func WithGzipRequest(r *request.Request) {
	r.Handlers.Build.PushBackNamed(gzip.NewGzipRequestHandler())
}
