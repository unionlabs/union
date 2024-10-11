package s3control

import (
	"net/url"
	"strings"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/internal/s3shared"
	"github.com/aws/aws-sdk-go/internal/s3shared/arn"
	"github.com/aws/aws-sdk-go/private/protocol"
)

const (
	accessPointPrefixLabel = "accesspoint"
	accountIDPrefixLabel   = "accountID"

	outpostPrefixLabel = "outpost"
)

// hasCustomEndpoint returns true if endpoint is a custom endpoint
func hasCustomEndpoint(r *request.Request) bool {
	return len(aws.StringValue(r.Config.Endpoint)) > 0
}

// outpostAccessPointEndpointBuilder represents the endpoint builder for outpost access point arn.
type outpostAccessPointEndpointBuilder arn.OutpostAccessPointARN

// build builds an endpoint corresponding to the outpost access point arn.
//
// For building an endpoint from outpost access point arn, format used is:
// - Outpost access point endpoint format : s3-outposts.{region}.{dnsSuffix}
// - example : s3-outposts.us-west-2.amazonaws.com
//
// Outpost AccessPoint Endpoint request are signed using "s3-outposts" as signing name.
func (o outpostAccessPointEndpointBuilder) build(req *request.Request) error {
	resolveRegion := o.Region
	resolveService := o.Service

	endpointsID := resolveService
	if resolveService == "s3-outposts" {
		endpointsID = "s3"
	}

	endpoint, err := resolveRegionalEndpoint(req, resolveRegion, "", endpointsID)
	if err != nil {
		return s3shared.NewFailedToResolveEndpointError(o,
			req.ClientInfo.PartitionID, resolveRegion, err)
	}

	endpoint.URL = endpoints.AddScheme(endpoint.URL, aws.BoolValue(req.Config.DisableSSL))

	if !hasCustomEndpoint(req) {
		if err = updateRequestEndpoint(req, endpoint.URL); err != nil {
			return err
		}
		// add url host as s3-outposts
		cfgHost := req.HTTPRequest.URL.Host
		if strings.HasPrefix(cfgHost, endpointsID) {
			req.HTTPRequest.URL.Host = resolveService + cfgHost[len(endpointsID):]
		}
	}

	// set the signing region, name to resolved names from ARN
	redirectSigner(req, resolveService, resolveRegion)

	err = protocol.ValidateEndpointHost(req.Operation.Name, req.HTTPRequest.URL.Host)
	if err != nil {
		return s3shared.NewInvalidARNError(o, err)
	}

	return nil
}

func (o outpostAccessPointEndpointBuilder) hostPrefixLabelValues() map[string]string {
	return map[string]string{
		accessPointPrefixLabel: o.AccessPointName,
		accountIDPrefixLabel:   o.AccountID,
		outpostPrefixLabel:     o.OutpostID,
	}
}

// outpostBucketResourceEndpointBuilder represents the endpoint builder for outpost bucket resource arn
type outpostBucketResourceEndpointBuilder arn.OutpostBucketARN

// build builds the endpoint for corresponding outpost bucket arn
//
// For building an endpoint from outpost bucket arn, format used is:
// - Outpost bucket arn endpoint format : s3-outposts.{region}.{dnsSuffix}
// - example : s3-outposts.us-west-2.amazonaws.com
//
// Outpost bucket arn endpoint request are signed using "s3-outposts" as signing name
func (o outpostBucketResourceEndpointBuilder) build(req *request.Request) error {
	resolveService := arn.OutpostBucketARN(o).Service
	resolveRegion := arn.OutpostBucketARN(o).Region

	// Outpost bucket resource uses `s3-control` as serviceEndpointLabel
	endpointsID := "s3-control"

	endpoint, err := resolveRegionalEndpoint(req, resolveRegion, "", endpointsID)
	if err != nil {
		return s3shared.NewFailedToResolveEndpointError(arn.OutpostBucketARN(o),
			req.ClientInfo.PartitionID, resolveRegion, err)
	}

	endpoint.URL = endpoints.AddScheme(endpoint.URL, aws.BoolValue(req.Config.DisableSSL))

	if !hasCustomEndpoint(req) {
		if err = updateRequestEndpoint(req, endpoint.URL); err != nil {
			return err
		}

		// add url host as s3-outposts
		cfgHost := req.HTTPRequest.URL.Host
		if strings.HasPrefix(cfgHost, endpointsID) {
			req.HTTPRequest.URL.Host = resolveService + cfgHost[len(endpointsID):]
		}
	}

	// signer redirection
	redirectSigner(req, resolveService, resolveRegion)

	err = protocol.ValidateEndpointHost(req.Operation.Name, req.HTTPRequest.URL.Host)
	if err != nil {
		return s3shared.NewInvalidARNError(arn.OutpostBucketARN(o), err)
	}
	return nil
}

func resolveRegionalEndpoint(r *request.Request, region, resolvedRegion, endpointsID string) (endpoints.ResolvedEndpoint, error) {
	return r.Config.EndpointResolver.EndpointFor(endpointsID, region, func(opts *endpoints.Options) {
		opts.DisableSSL = aws.BoolValue(r.Config.DisableSSL)
		opts.UseDualStack = aws.BoolValue(r.Config.UseDualStack)
		opts.UseDualStackEndpoint = r.Config.UseDualStackEndpoint
		opts.UseFIPSEndpoint = r.Config.UseFIPSEndpoint
		opts.S3UsEast1RegionalEndpoint = endpoints.RegionalS3UsEast1Endpoint
		opts.ResolvedRegion = resolvedRegion
		opts.Logger = r.Config.Logger
		opts.LogDeprecated = r.Config.LogLevel.Matches(aws.LogDebugWithDeprecated)
	})
}

func updateRequestEndpoint(r *request.Request, endpoint string) (err error) {
	r.HTTPRequest.URL, err = url.Parse(endpoint + r.Operation.HTTPPath)
	if err != nil {
		return awserr.New(request.ErrCodeSerialization,
			"failed to parse endpoint URL", err)
	}

	return nil
}

// redirectSigner sets signing name, signing region for a request
func redirectSigner(req *request.Request, signingName string, signingRegion string) {
	req.ClientInfo.SigningName = signingName
	req.ClientInfo.SigningRegion = signingRegion
}
