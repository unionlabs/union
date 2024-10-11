//go:build codegen
// +build codegen

package api

type legacyJSONValues struct {
	Type          string
	StructMembers map[string]struct{}
	ListMemberRef bool
	MapValueRef   bool
}

var legacyJSONValueShapes = map[string]map[string]legacyJSONValues{
	"braket": map[string]legacyJSONValues{
		"CreateQuantumTaskRequest": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"action":           struct{}{},
				"deviceParameters": struct{}{},
			},
		},
		"GetDeviceResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"deviceCapabilities": struct{}{},
			},
		},
		"GetQuantumTaskResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"deviceParameters": struct{}{},
			},
		},
	},
	"cloudwatchrum": map[string]legacyJSONValues{
		"RumEvent": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"details":  struct{}{},
				"metadata": struct{}{},
			},
		},
	},
	"lexruntimeservice": map[string]legacyJSONValues{
		"PostContentRequest": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"requestAttributes": struct{}{},
				//"ActiveContexts":    struct{}{}, - Disabled because JSON List
				"sessionAttributes": struct{}{},
			},
		},
		"PostContentResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				// "alternativeIntents":  struct{}{}, - Disabled because JSON List
				"sessionAttributes":   struct{}{},
				"nluIntentConfidence": struct{}{},
				"slots":               struct{}{},
				//"activeContexts":      struct{}{}, - Disabled because JSON List
			},
		},
		"PutSessionResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				// "activeContexts":    struct{}{}, - Disabled because JSON List
				"slots":             struct{}{},
				"sessionAttributes": struct{}{},
			},
		},
	},
	"lookoutequipment": map[string]legacyJSONValues{
		"DatasetSchema": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"InlineDataSchema": struct{}{},
			},
		},
		"DescribeDatasetResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Schema": struct{}{},
			},
		},
		"DescribeModelResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Schema":       struct{}{},
				"ModelMetrics": struct{}{},
			},
		},
	},
	"networkmanager": map[string]legacyJSONValues{
		"CoreNetworkPolicy": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"PolicyDocument": struct{}{},
			},
		},
		"GetResourcePolicyResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"PolicyDocument": struct{}{},
			},
		},
		"PutCoreNetworkPolicyRequest": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"PolicyDocument": struct{}{},
			},
		},
		"PutResourcePolicyRequest": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"PolicyDocument": struct{}{},
			},
		},
	},
	"personalizeevents": map[string]legacyJSONValues{
		"Event": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"properties": struct{}{},
			},
		},
		"Item": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"properties": struct{}{},
			},
		},
		"User": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"properties": struct{}{},
			},
		},
	},
	"pricing": map[string]legacyJSONValues{
		"PriceListJsonItems": legacyJSONValues{
			Type:          "list",
			ListMemberRef: true,
		},
	},
	"rekognition": map[string]legacyJSONValues{
		"HumanLoopActivationOutput": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"HumanLoopActivationConditionsEvaluationResults": struct{}{},
			},
		},
	},
	"sagemaker": map[string]legacyJSONValues{
		"HumanLoopActivationConditionsConfig": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"HumanLoopActivationConditions": struct{}{},
			},
		},
	},
	"schemas": map[string]legacyJSONValues{
		"GetResourcePolicyResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
		"PutResourcePolicyRequest": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
		"PutResourcePolicyResponse": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
		"GetResourcePolicyOutput": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
		"PutResourcePolicyInput": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
		"PutResourcePolicyOutput": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"Policy": struct{}{},
			},
		},
	},
	"textract": map[string]legacyJSONValues{
		"HumanLoopActivationOutput": legacyJSONValues{
			Type: "structure",
			StructMembers: map[string]struct{}{
				"HumanLoopActivationConditionsEvaluationResults": struct{}{},
			},
		},
	},
}
