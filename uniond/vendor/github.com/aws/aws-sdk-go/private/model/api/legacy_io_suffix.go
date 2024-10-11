//go:build codegen
// +build codegen

package api

// IoSuffix represents map of service to shape names that
// are suffixed with `Input`, `Output` string and are not
// Input or Output shapes used by any operation within
// the service enclosure.
type IoSuffix map[string]map[string]struct{}

// LegacyIoSuffix returns if the shape names are legacy
// names that contain "Input" and "Output" name as suffix.
func (i IoSuffix) LegacyIOSuffix(a *API, shapeName string) bool {
	names, ok := i[a.name]
	if !ok {
		return false
	}

	_, ok = names[shapeName]
	return ok
}

// legacyIOSuffixed is the list of known shapes that have "Input" and "Output"
// as suffix in shape name, but are not the actual input, output shape
// for a corresponding service operation.
var legacyIOSuffixed = IoSuffix{
	"TranscribeService": {
		"RedactionOutput": struct{}{},
	},
	"Textract": {"HumanLoopActivationOutput": struct{}{}},
	"Synthetics": {
		"CanaryRunConfigInput":  struct{}{},
		"CanaryScheduleOutput":  struct{}{},
		"VpcConfigInput":        struct{}{},
		"VpcConfigOutput":       struct{}{},
		"CanaryCodeOutput":      struct{}{},
		"CanaryCodeInput":       struct{}{},
		"CanaryRunConfigOutput": struct{}{},
		"CanaryScheduleInput":   struct{}{},
	},
	"SWF": {"FunctionInput": struct{}{}},
	"SFN": {
		"InvalidOutput":         struct{}{},
		"InvalidExecutionInput": struct{}{},
		"SensitiveDataJobInput": struct{}{},
	},
	"SSM": {
		"CommandPluginOutput":                 struct{}{},
		"MaintenanceWindowStepFunctionsInput": struct{}{},
		"InvocationTraceOutput":               struct{}{},
	},
	"SSMIncidents": {"RegionMapInput": struct{}{}},

	"SMS": {
		"AppValidationOutput":    struct{}{},
		"ServerValidationOutput": struct{}{},
		"ValidationOutput":       struct{}{},
		"SSMOutput":              struct{}{},
	},
	"ServiceDiscovery": {"InvalidInput": struct{}{}},
	"ServiceCatalog": {
		"RecordOutput":               struct{}{},
		"ProvisioningArtifactOutput": struct{}{},
	},
	"Schemas": {
		"GetDiscoveredSchemaVersionItemInput":         struct{}{},
		"__listOfGetDiscoveredSchemaVersionItemInput": struct{}{},
	},

	"SageMaker": {
		"ProcessingOutput":             struct{}{},
		"TaskInput":                    struct{}{},
		"TransformOutput":              struct{}{},
		"ModelBiasJobInput":            struct{}{},
		"TransformInput":               struct{}{},
		"LabelingJobOutput":            struct{}{},
		"DataQualityJobInput":          struct{}{},
		"MonitoringOutput":             struct{}{},
		"MonitoringS3Output":           struct{}{},
		"MonitoringInput":              struct{}{},
		"ProcessingS3Output":           struct{}{},
		"ModelQualityJobInput":         struct{}{},
		"ProcessingInput":              struct{}{},
		"ProcessingFeatureStoreOutput": struct{}{},
		"ModelExplainabilityJobInput":  struct{}{},
		"ProcessingS3Input":            struct{}{},
		"MonitoringGroundTruthS3Input": struct{}{},
		"EdgePresetDeploymentOutput":   struct{}{},
		"EndpointInput":                struct{}{},
	},

	"AugmentedAIRuntime": {"HumanLoopOutput": struct{}{}, "HumanLoopInput": struct{}{}},

	"S3": {
		"ParquetInput": struct{}{},
		"CSVOutput":    struct{}{},
		"JSONOutput":   struct{}{},
		"JSONInput":    struct{}{},
		"CSVInput":     struct{}{},
	},

	"Route53Domains": {"InvalidInput": struct{}{}},
	"Route53":        {"InvalidInput": struct{}{}},
	"RoboMaker":      {"S3KeyOutput": struct{}{}},

	"Rekognition": {
		"StreamProcessorInput":      struct{}{},
		"HumanLoopActivationOutput": struct{}{},
		"StreamProcessorOutput":     struct{}{},
	},

	"Proton": {"TemplateVersionSourceInput": struct{}{}, "CompatibleEnvironmentTemplateInput": struct{}{}},

	"Personalize": {
		"BatchInferenceJobInput":  struct{}{},
		"BatchInferenceJobOutput": struct{}{},
		"DatasetExportJobOutput":  struct{}{},
	},

	"MWAA": {
		"ModuleLoggingConfigurationInput": struct{}{},
		"LoggingConfigurationInput":       struct{}{},
		"UpdateNetworkConfigurationInput": struct{}{},
	},

	"MQ": {"LdapServerMetadataOutput": struct{}{}, "LdapServerMetadataInput": struct{}{}},

	"MediaLive": {
		"InputDeviceConfiguredInput": struct{}{},
		"__listOfOutput":             struct{}{},
		"Input":                      struct{}{},
		"__listOfInput":              struct{}{},
		"Output":                     struct{}{},
		"InputDeviceActiveInput":     struct{}{},
	},

	"MediaConvert": {
		"Input":          struct{}{},
		"__listOfOutput": struct{}{},
		"Output":         struct{}{},
		"__listOfInput":  struct{}{},
	},
	"MediaConnect": {"Output": struct{}{}, "__listOfOutput": struct{}{}},

	"Lambda": {
		"LayerVersionContentOutput": struct{}{},
		"LayerVersionContentInput":  struct{}{},
	},

	"KinesisAnalyticsV2": {
		"KinesisStreamsInput":   struct{}{},
		"KinesisFirehoseInput":  struct{}{},
		"LambdaOutput":          struct{}{},
		"Output":                struct{}{},
		"KinesisFirehoseOutput": struct{}{},
		"Input":                 struct{}{},
		"KinesisStreamsOutput":  struct{}{},
	},

	"KinesisAnalytics": {
		"Output":                struct{}{},
		"KinesisFirehoseInput":  struct{}{},
		"LambdaOutput":          struct{}{},
		"KinesisFirehoseOutput": struct{}{},
		"KinesisStreamsInput":   struct{}{},
		"Input":                 struct{}{},
		"KinesisStreamsOutput":  struct{}{},
	},

	"IoTEvents": {"Input": struct{}{}},
	"IoT":       {"PutItemInput": struct{}{}},

	"Honeycode": {"CellInput": struct{}{}, "RowDataInput": struct{}{}},

	"Glue": {
		"TableInput":               struct{}{},
		"UserDefinedFunctionInput": struct{}{},
		"DatabaseInput":            struct{}{},
		"PartitionInput":           struct{}{},
		"ConnectionInput":          struct{}{},
	},

	"Glacier": {
		"CSVInput":                   struct{}{},
		"CSVOutput":                  struct{}{},
		"InventoryRetrievalJobInput": struct{}{},
	},

	"FIS": {
		"CreateExperimentTemplateTargetInput":        struct{}{},
		"CreateExperimentTemplateStopConditionInput": struct{}{},
		"UpdateExperimentTemplateStopConditionInput": struct{}{},
		"CreateExperimentTemplateActionInput":        struct{}{},
		"UpdateExperimentTemplateTargetInput":        struct{}{},
	},

	"Firehose": {"DeliveryStreamEncryptionConfigurationInput": struct{}{}},

	"CloudWatchEvents": {"TransformerInput": struct{}{}, "TargetInput": struct{}{}},

	"EventBridge": {"TransformerInput": struct{}{}, "TargetInput": struct{}{}},

	"ElasticsearchService": {
		"AutoTuneOptionsOutput":        struct{}{},
		"SAMLOptionsInput":             struct{}{},
		"AdvancedSecurityOptionsInput": struct{}{},
		"SAMLOptionsOutput":            struct{}{},
		"AutoTuneOptionsInput":         struct{}{},
	},

	"ElasticTranscoder": {
		"JobOutput":       struct{}{},
		"CreateJobOutput": struct{}{},
		"JobInput":        struct{}{},
	},

	"ElastiCache": {
		"UserGroupIdListInput": struct{}{},
		"PasswordListInput":    struct{}{},
		"UserIdListInput":      struct{}{},
	},

	"ECRPublic":  {"RepositoryCatalogDataInput": struct{}{}},
	"DeviceFarm": {"TestGridUrlExpiresInSecondsInput": struct{}{}},

	"GlueDataBrew": {"Output": struct{}{}, "Input": struct{}{}, "OverwriteOutput": struct{}{}},

	"CodePipeline": {"ActionExecutionInput": struct{}{}, "ActionExecutionOutput": struct{}{}},

	"CodeBuild": {"ValueInput": struct{}{}, "KeyInput": struct{}{}},

	"CloudFormation": {"Output": struct{}{}},

	"Backup": {
		"PlanInput":  struct{}{},
		"RulesInput": struct{}{},
		"RuleInput":  struct{}{},
	},

	"ApplicationInsights": {"StatesInput": struct{}{}},

	"ApiGatewayV2": {
		"TlsConfigInput":               struct{}{},
		"MutualTlsAuthenticationInput": struct{}{},
	},
	"APIGateway": {"MutualTlsAuthenticationInput": struct{}{}},
}
