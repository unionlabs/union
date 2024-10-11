//go:build codegen
// +build codegen

package api

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"
)

type service struct {
	srcName string
	dstName string

	serviceVersion string
}

var mergeServices = map[string]service{
	"dynamodbstreams": {
		dstName: "dynamodb",
		srcName: "streams.dynamodb",
	},
	"wafregional": {
		dstName:        "waf",
		srcName:        "waf-regional",
		serviceVersion: "2015-08-24",
	},
}

var serviceAliaseNames = map[string]string{
	"costandusagereportservice": "CostandUsageReportService",
	"elasticloadbalancing":      "ELB",
	"elasticloadbalancingv2":    "ELBV2",
	"config":                    "ConfigService",
}

func (a *API) setServiceAliaseName() {
	if newName, ok := serviceAliaseNames[a.PackageName()]; ok {
		a.name = newName
	}
}

// customizationPasses Executes customization logic for the API by package name.
func (a *API) customizationPasses() error {
	var svcCustomizations = map[string]func(*API) error{
		"s3":         s3Customizations,
		"s3control":  s3ControlCustomizations,
		"cloudfront": cloudfrontCustomizations,
		"rds":        rdsCustomizations,
		"neptune":    neptuneCustomizations,
		"docdb":      docdbCustomizations,

		// Disable endpoint resolving for services that require customer
		// to provide endpoint them selves.
		"cloudsearchdomain": disableEndpointResolving,
		"iotdataplane":      disableEndpointResolving,

		// MTurk smoke test is invalid. The service requires AWS account to be
		// linked to Amazon Mechanical Turk Account.
		"mturk": supressSmokeTest,

		// Backfill the authentication type for cognito identity and sts.
		// Removes the need for the customizations in these services.
		"cognitoidentity": backfillAuthType(NoneAuthType,
			"GetId",
			"GetOpenIdToken",
			"UnlinkIdentity",
			"GetCredentialsForIdentity",
		),
		"sts": backfillAuthType(NoneAuthType,
			"AssumeRoleWithSAML",
			"AssumeRoleWithWebIdentity",
		),
		"eventbridge": eventBridgeCustomizations,
	}

	for k := range mergeServices {
		svcCustomizations[k] = mergeServicesCustomizations
	}

	if fn := svcCustomizations[a.PackageName()]; fn != nil {
		err := fn(a)
		if err != nil {
			return fmt.Errorf("service customization pass failure for %s: %v", a.PackageName(), err)
		}
	}

	if err := addHTTPChecksumCustomDocumentation(a); err != nil {
		if err != nil {
			return fmt.Errorf("service httpChecksum trait customization failed, %s: %v",
				a.PackageName(), err)
		}
	}

	return nil
}

func addHTTPChecksumCustomDocumentation(a *API) error {
	for opName, o := range a.Operations {
		if o.HTTPChecksum.RequestAlgorithmMember != "" {
			ref := o.InputRef.Shape.GetModeledMember(o.HTTPChecksum.RequestAlgorithmMember)
			if ref == nil {
				return fmt.Errorf(
					"expect httpChecksum.RequestAlgorithmMember %v to be modeled input member for %v",
					o.HTTPChecksum.RequestAlgorithmMember,
					opName,
				)
			}

			ref.Documentation = AppendDocstring(ref.Documentation, `
				The AWS SDK for Go v1 does not support automatic computing
				request payload checksum. This feature is available in the AWS
				SDK for Go v2. If a value is specified for this parameter, the
				matching algorithm's checksum member must be populated with the
				algorithm's checksum of the request payload. 
			`)
			if o.RequestChecksumRequired() {
				ref.Documentation = AppendDocstring(ref.Documentation, `
					The SDK will automatically compute the Content-MD5 checksum
					for this operation. The AWS SDK for Go v2 allows you to
					configure alternative checksum algorithm to be used.
				`)
			}
		}

		if o.HTTPChecksum.RequestValidationModeMember != "" {
			ref := o.InputRef.Shape.GetModeledMember(o.HTTPChecksum.RequestValidationModeMember)
			if ref == nil {
				return fmt.Errorf(
					"expect httpChecksum.RequestValidationModeMember %v to be modeled input member for %v",
					o.HTTPChecksum.RequestValidationModeMember,
					opName,
				)
			}

			ref.Documentation = AppendDocstring(ref.Documentation, `
				The AWS SDK for Go v1 does not support automatic response
				payload checksum validation. This feature is available in the
				AWS SDK for Go v2.
			`)
		}
	}

	return nil
}

func eventBridgeCustomizations(a *API) error {
	// Inject documentation to indicate PutEvents API does not support EndpointId routing to a multi-region endpoint
	// using SigV4a signing.

	const docAddon = "// This AWS SDK does not support calling multi-region endpoints with SigV4a authentication."

	var sb strings.Builder

	op, ok := a.Operations["PutEvents"]
	if !ok {
		return nil
	}

	op.Documentation = appendDocString(&sb, op.Documentation, docAddon)

	const putEventsInputShape = "PutEventsInput"
	input, ok := a.Shapes[putEventsInputShape]
	if !ok {
		return nil
	}

	const endpointIdMember = "EndpointId"
	mref, ok := input.MemberRefs[endpointIdMember]
	if !ok {
		return nil
	}

	mref.Documentation = appendDocString(&sb, mref.Documentation, docAddon)

	return nil
}

func appendDocString(sb *strings.Builder, doc, content string) string {
	if len(content) == 0 {
		return doc
	}
	sb.Reset()
	sb.WriteString(doc)
	if sb.Len() > 0 {
		if doc[len(doc)-1] != '\n' {
			sb.WriteRune('\n')
		}
		sb.WriteString("//\n")
	}
	sb.WriteString(content)
	return sb.String()
}

func supressSmokeTest(a *API) error {
	a.SmokeTests.TestCases = []SmokeTestCase{}
	return nil
}

// Customizes the API generation to replace values specific to S3.
func s3Customizations(a *API) error {

	// back-fill signing name as 's3'
	a.Metadata.SigningName = "s3"

	var strExpires *Shape

	var keepContentMD5Ref = map[string]struct{}{
		"PutObjectInput":  {},
		"UploadPartInput": {},
	}

	for name, s := range a.Shapes {
		// Remove ContentMD5 members unless specified otherwise.
		if _, keep := keepContentMD5Ref[name]; !keep {
			if _, have := s.MemberRefs["ContentMD5"]; have {
				delete(s.MemberRefs, "ContentMD5")
			}
		}

		// Generate getter methods for API operation fields used by customizations.
		for _, refName := range []string{"Bucket", "SSECustomerKey", "CopySourceSSECustomerKey"} {
			if ref, ok := s.MemberRefs[refName]; ok {
				ref.GenerateGetter = true
			}
		}

		// Generate a endpointARN method for the BucketName shape if this is used as an operation input
		if s.UsedAsInput {
			if s.ShapeName == "CreateBucketInput" {
				// For all operations but CreateBucket the BucketName shape
				// needs to be decorated.
				continue
			}
			var endpointARNShape *ShapeRef
			for _, ref := range s.MemberRefs {
				if ref.OrigShapeName != "BucketName" || ref.Shape.Type != "string" {
					continue
				}
				if endpointARNShape != nil {
					return fmt.Errorf("more then one BucketName shape present on shape")
				}
				ref.EndpointARN = true
				endpointARNShape = ref
			}
			if endpointARNShape != nil {
				s.HasEndpointARNMember = true
				a.HasEndpointARN = true
			}
		}

		// Decorate member references that are modeled with the wrong type.
		// Specifically the case where a member was modeled as a string, but is
		// expected to sent across the wire as a base64 value.
		//
		// e.g. S3's SSECustomerKey and CopySourceSSECustomerKey
		for _, refName := range []string{
			"SSECustomerKey",
			"CopySourceSSECustomerKey",
		} {
			if ref, ok := s.MemberRefs[refName]; ok {
				ref.CustomTags = append(ref.CustomTags, ShapeTag{
					"marshal-as", "blob",
				})
			}
		}

		// Expires should be a string not time.Time since the format is not
		// enforced by S3, and any value can be set to this field outside of the SDK.
		if strings.HasSuffix(name, "Output") {
			if ref, ok := s.MemberRefs["Expires"]; ok {
				if strExpires == nil {
					newShape := *ref.Shape
					strExpires = &newShape
					strExpires.Type = "string"
					strExpires.refs = []*ShapeRef{}
				}
				ref.Shape.removeRef(ref)
				ref.Shape = strExpires
				ref.Shape.refs = append(ref.Shape.refs, &s.MemberRef)
			}
		}
	}
	s3CustRemoveHeadObjectModeledErrors(a)

	return nil
}

// S3 HeadObject API call incorrect models NoSuchKey as valid
// error code that can be returned. This operation does not
// return error codes, all error codes are derived from HTTP
// status codes.
//
// aws/aws-sdk-go#1208
func s3CustRemoveHeadObjectModeledErrors(a *API) {
	op, ok := a.Operations["HeadObject"]
	if !ok {
		return
	}
	op.Documentation = AppendDocstring(op.Documentation, `
		See http://docs.aws.amazon.com/AmazonS3/latest/API/ErrorResponses.html#RESTErrorResponses
		for more information on returned errors.
	`)
	op.ErrorRefs = []ShapeRef{}
}

// S3 service operations with an AccountId need accessors to be generated for
// them so the fields can be dynamically accessed without reflection.
func s3ControlCustomizations(a *API) error {
	for _, s := range a.Shapes {
		// Generate a endpointARN method for the BucketName shape if this is used as an operation input
		if s.UsedAsInput {
			if s.ShapeName == "CreateBucketInput" || s.ShapeName == "ListRegionalBucketsInput" {
				// For operations CreateBucketInput and ListRegionalBuckets the OutpostID shape
				// needs to be decorated
				var outpostIDMemberShape *ShapeRef
				for memberName, ref := range s.MemberRefs {
					if memberName != "OutpostId" || ref.Shape.Type != "string" {
						continue
					}
					if outpostIDMemberShape != nil {
						return fmt.Errorf("more then one OutpostID shape present on shape")
					}
					ref.OutpostIDMember = true
					outpostIDMemberShape = ref
				}
				if outpostIDMemberShape != nil {
					s.HasOutpostIDMember = true
					a.HasOutpostID = true
				}
				continue
			}

			// List of input shapes that use accesspoint names as arnable fields
			accessPointNameArnables := map[string]struct{}{
				"GetAccessPointInput":          {},
				"DeleteAccessPointInput":       {},
				"PutAccessPointPolicyInput":    {},
				"GetAccessPointPolicyInput":    {},
				"DeleteAccessPointPolicyInput": {},
			}

			var endpointARNShape *ShapeRef
			for _, ref := range s.MemberRefs {
				// Operations that have AccessPointName field that takes in an ARN as input
				if _, ok := accessPointNameArnables[s.ShapeName]; ok {
					if ref.OrigShapeName != "AccessPointName" || ref.Shape.Type != "string" {
						continue
					}
				} else if ref.OrigShapeName != "BucketName" || ref.Shape.Type != "string" {
					// All other operations currently allow BucketName field to take in ARN.
					// Exceptions for these are CreateBucket and ListRegionalBucket which use
					// Outpost id and are handled above separately.
					continue
				}

				if endpointARNShape != nil {
					return fmt.Errorf("more then one member present on shape takes arn as input")
				}
				ref.EndpointARN = true
				endpointARNShape = ref
			}
			if endpointARNShape != nil {
				s.HasEndpointARNMember = true
				a.HasEndpointARN = true

				for _, ref := range s.MemberRefs {
					// check for account id customization
					if ref.OrigShapeName == "AccountId" && ref.Shape.Type == "string" {
						ref.AccountIDMemberWithARN = true
						s.HasAccountIdMemberWithARN = true
						a.HasAccountIdWithARN = true
					}
				}
			}
		}
	}

	return nil
}

// cloudfrontCustomizations customized the API generation to replace values
// specific to CloudFront.
func cloudfrontCustomizations(a *API) error {
	// MaxItems members should always be integers
	for _, s := range a.Shapes {
		if ref, ok := s.MemberRefs["MaxItems"]; ok {
			ref.ShapeName = "Integer"
			ref.Shape = a.Shapes["Integer"]
		}
	}
	return nil
}

// mergeServicesCustomizations references any duplicate shapes from DynamoDB
func mergeServicesCustomizations(a *API) error {
	info := mergeServices[a.PackageName()]

	p := strings.Replace(a.path, info.srcName, info.dstName, -1)

	if info.serviceVersion != "" {
		index := strings.LastIndex(p, string(filepath.Separator))
		files, _ := ioutil.ReadDir(p[:index])
		if len(files) > 1 {
			panic("New version was introduced")
		}
		p = p[:index] + "/" + info.serviceVersion
	}

	file := filepath.Join(p, "api-2.json")

	serviceAPI := API{
		IgnoreUnsupportedAPIs:        a.IgnoreUnsupportedAPIs,
		NoRemoveUnusedShapes:         a.NoRemoveUnusedShapes,
		NoRenameToplevelShapes:       a.NoRenameToplevelShapes,
		NoInitMethods:                a.NoInitMethods,
		NoStringerMethods:            a.NoStringerMethods,
		NoConstServiceNames:          a.NoConstServiceNames,
		NoValidataShapeMethods:       a.NoValidataShapeMethods,
		NoGenStructFieldAccessors:    a.NoGenStructFieldAccessors,
		NoRemoveUnsupportedJSONValue: a.NoRemoveUnsupportedJSONValue,
		StrictServiceId:              a.StrictServiceId,
	}
	serviceAPI.Attach(file)
	serviceAPI.Setup()

	for n := range a.Shapes {
		if _, ok := serviceAPI.Shapes[n]; ok {
			a.Shapes[n].resolvePkg = SDKImportRoot + "/service/" + info.dstName
		}
	}

	return nil
}

// rdsCustomizations are customization for the service/rds. This adds
// non-modeled fields used for presigning.
func rdsCustomizations(a *API) error {
	inputs := []string{
		"CopyDBSnapshotInput",
		"CreateDBInstanceReadReplicaInput",
		"CopyDBClusterSnapshotInput",
		"CreateDBClusterInput",
		"StartDBInstanceAutomatedBackupsReplicationInput",
	}
	generatePresignedURL(a, inputs)
	return nil
}

// neptuneCustomizations are customization for the service/neptune. This adds
// non-modeled fields used for presigning.
func neptuneCustomizations(a *API) error {
	inputs := []string{
		"CopyDBClusterSnapshotInput",
		"CreateDBClusterInput",
	}
	generatePresignedURL(a, inputs)
	return nil
}

// neptuneCustomizations are customization for the service/neptune. This adds
// non-modeled fields used for presigning.
func docdbCustomizations(a *API) error {
	inputs := []string{
		"CopyDBClusterSnapshotInput",
		"CreateDBClusterInput",
	}
	generatePresignedURL(a, inputs)
	return nil
}

func generatePresignedURL(a *API, inputShapes []string) {
	for _, input := range inputShapes {
		if ref, ok := a.Shapes[input]; ok {
			ref.MemberRefs["SourceRegion"] = &ShapeRef{
				Documentation: docstring(`
				SourceRegion is the source region where the resource exists.
				This is not sent over the wire and is only used for presigning.
				This value should always have the same region as the source
				ARN.
				`),
				ShapeName: "String",
				Shape:     a.Shapes["String"],
				Ignore:    true,
			}
			ref.MemberRefs["DestinationRegion"] = &ShapeRef{
				Documentation: docstring(`
				DestinationRegion is used for presigning the request to a given region.
				`),
				ShapeName: "String",
				Shape:     a.Shapes["String"],
			}
		}
	}
}

func disableEndpointResolving(a *API) error {
	a.Metadata.NoResolveEndpoint = true
	return nil
}

func backfillAuthType(typ AuthType, opNames ...string) func(*API) error {
	return func(a *API) error {
		for _, opName := range opNames {
			op, ok := a.Operations[opName]
			if !ok {
				panic("unable to backfill auth-type for unknown operation " + opName)
			}
			if v := op.AuthType; len(v) != 0 {
				fmt.Fprintf(os.Stderr, "unable to backfill auth-type for %s, already set, %s\n", opName, v)
				continue
			}

			op.AuthType = typ
		}

		return nil
	}
}

// Must be invoked with the original shape name
func removeUnsupportedJSONValue(a *API) error {
	for shapeName, shape := range a.Shapes {
		switch shape.Type {
		case "structure":
			for refName, ref := range shape.MemberRefs {
				if !ref.JSONValue {
					continue
				}
				if err := removeUnsupportedShapeRefJSONValue(a, shapeName, refName, ref); err != nil {
					return fmt.Errorf("failed remove unsupported JSONValue from %v.%v, %v",
						shapeName, refName, err)
				}
			}
		case "list":
			if !shape.MemberRef.JSONValue {
				continue
			}
			if err := removeUnsupportedShapeRefJSONValue(a, shapeName, "", &shape.MemberRef); err != nil {
				return fmt.Errorf("failed remove unsupported JSONValue from %v, %v",
					shapeName, err)
			}
		case "map":
			if !shape.ValueRef.JSONValue {
				continue
			}
			if err := removeUnsupportedShapeRefJSONValue(a, shapeName, "", &shape.ValueRef); err != nil {
				return fmt.Errorf("failed remove unsupported JSONValue from %v, %v",
					shapeName, err)
			}
		}
	}

	return nil
}

func removeUnsupportedShapeRefJSONValue(a *API, parentName, refName string, ref *ShapeRef) (err error) {
	var found bool

	defer func() {
		if !found && err == nil {
			log.Println("removing JSONValue", a.PackageName(), parentName, refName)
			ref.JSONValue = false
			ref.SuppressedJSONValue = true
		}
	}()

	legacyShapes, ok := legacyJSONValueShapes[a.PackageName()]
	if !ok {
		return nil
	}

	legacyShape, ok := legacyShapes[parentName]
	if !ok {
		return nil
	}

	switch legacyShape.Type {
	case "structure":
		_, ok = legacyShape.StructMembers[refName]
		found = ok
	case "list":
		found = legacyShape.ListMemberRef
	case "map":
		found = legacyShape.MapValueRef
	}

	return nil
}
