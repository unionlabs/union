/*
unknownproto implements functionality to "type check" protobuf serialized byte sequences
against an expected proto.Message to report:

a) Unknown fields in the stream -- this is indicative of mismatched services, perhaps a malicious actor

b) Mismatched wire types for a field -- this is indicative of mismatched services

Its API signature is similar to proto.Unmarshal([]byte, proto.Message) in the strict case

	if err := RejectUnknownFieldsStrict(protoBlob, protoMessage, false); err != nil {
	        // Handle the error.
	}

and ideally should be added before invoking proto.Unmarshal, if you'd like to enforce the features mentioned above.

By default, for security we report every single field that's unknown, whether a non-critical field or not. To customize
this behavior, please set the boolean parameter allowUnknownNonCriticals to true to RejectUnknownFields:

	if err := RejectUnknownFields(protoBlob, protoMessage, true); err != nil {
	        // Handle the error.
	}
*/
package unknownproto
