// Manual code for logging field extraction tests.

package mwitkow_testproto

// This is implementing grpc_logging.requestLogFieldsExtractor
func (m *PingRequest) ExtractRequestFields(appendToMap map[string]interface{}) {
	appendToMap["value"] = m.Value
}
