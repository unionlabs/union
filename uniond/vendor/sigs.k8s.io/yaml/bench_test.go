/*
Copyright 2022 The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package yaml

import (
	"encoding/json"
	"fmt"
	"testing"

	"sigs.k8s.io/yaml/goyaml.v2"
)

func newBenchmarkObject() interface{} {
	data := struct {
		Object map[string]interface{}
		Items  []interface{}
	}{
		Object: map[string]interface{}{
			"apiVersion": "v1",
			"kind":       "PodList",
		},
		Items: []interface{}{},
	}
	for i := 0; i < 1000; i++ {
		item := struct {
			Object map[string]interface{}
		}{
			Object: map[string]interface{}{
				"apiVersion": "v1",
				"kind":       "Pod",
				"metadata": map[string]interface{}{
					"creationTimestamp": "2022-04-18T21:03:19Z",
					"labels": map[string]interface{}{
						"run": fmt.Sprintf("pod%d", i),
					},
					"name":            fmt.Sprintf("pod%d", i),
					"namespace":       "default",
					"resourceVersion": "27622089",
					"uid":             "e8fe9315-3bed-4bb6-a70a-fb697c60deda",
				},
				"spec": map[string]interface{}{
					"containers": map[string]interface{}{
						"args": []string{
							"nc",
							"-lk",
							"-p",
							"8080",
							"-e",
							"cat",
						},
						"image":                    "busybox",
						"imagePullPolicy":          "Always",
						"name":                     "echo",
						"resources":                map[string]interface{}{},
						"terminationMessagePath":   "/dev/termination-log",
						"terminationMessagePolicy": "File",
						"volumeMounts": map[string]interface{}{
							"mountPath": "/var/run/secrets/kubernetes.io/serviceaccount",
							"name":      "kube-api-access-cpxzb",
							"readOnly":  true,
						},
					},
					"dnsPolicy":                     "ClusterFirst",
					"enableServiceLinks":            true,
					"nodeName":                      "k8s-worker-1",
					"preemptionPolicy":              "PreemptLowerPriority",
					"priority":                      0,
					"restartPolicy":                 "Always",
					"schedulerName":                 "default-scheduler",
					"securityContext":               map[string]interface{}{},
					"serviceAccount":                "default",
					"serviceAccountName":            "default",
					"terminationGracePeriodSeconds": 30,
					"tolerations": []map[string]interface{}{
						{
							"effect":            "NoExecute",
							"key":               "node.kubernetes.io/not-ready",
							"operator":          "Exists",
							"tolerationSeconds": 300,
						},
						{
							"effect":            "NoExecute",
							"key":               "node.kubernetes.io/unreachable",
							"operator":          "Exists",
							"tolerationSeconds": 300,
						},
					},
					"volumes": []map[string]interface{}{
						{
							"name": "kube-api-access-cpxzb",
							"projected": map[string]interface{}{
								"defaultMode": 420,
								"sources": []map[string]interface{}{
									{
										"serviceAccountToken": map[string]interface{}{
											"expirationSeconds": 3607,
											"path":              "token",
										},
									},
									{
										"configMap": map[string]interface{}{
											"items": []map[string]interface{}{
												{
													"key":  "ca.crt",
													"path": "ca.crt",
												},
											},
											"name": "kube-root-ca.crt",
										},
									},
									{
										"downwardAPI": map[string]interface{}{
											"items": []map[string]interface{}{
												{
													"fieldRef": map[string]interface{}{
														"apiVersion": "v1",
														"fieldPath":  "metadata.namespace",
													},
													"path": "namespace",
												},
											},
										},
									},
								},
							},
						},
					},
					"status": map[string]interface{}{
						"conditions": []map[string]interface{}{
							{
								"lastProbeTime":      nil,
								"lastTransitionTime": "2022-04-18T21:03:19Z",
								"status":             "True",
								"type":               "Initialized",
							},
							{
								"lastProbeTime":      nil,
								"lastTransitionTime": "2022-04-18T21:03:20Z",
								"status":             "True",
								"type":               "Ready",
							},
							{
								"lastProbeTime":      nil,
								"lastTransitionTime": "2022-04-18T21:03:20Z",
								"status":             "True",
								"type":               "ContainersReady",
							},
							{
								"lastProbeTime":      nil,
								"lastTransitionTime": "2022-04-18T21:03:19Z",
								"status":             "True",
								"type":               "PodScheduled",
							},
						},
						"containerStatuses": []map[string]interface{}{
							{
								"containerID":  "containerd://ed8afc051a21749e911a4dd4671e520dc81c8e1424853b6254872a3f461bb157",
								"image":        "docker.io/library/busybox:latest",
								"imageID":      "docker.io/library/busybox@sha256:d2b53584f580310186df7a2055ce3ff83cc0df6caacf1e3489bff8cf5d0af5d8",
								"lastState":    map[string]interface{}{},
								"name":         "echo",
								"ready":        true,
								"restartCount": 0,
								"started":      true,
								"state": map[string]interface{}{
									"running": map[string]interface{}{
										"startedAt": "2022-04-18T21:03:20Z",
									},
								},
							},
						},
						"hostIP": "192.168.200.12",
						"phase":  "Running",
						"podIP":  "10.244.1.248",
						"podIPs": []map[string]interface{}{
							{
								"ip": "10.244.1.248",
							},
						},
						"qosClass":  "BestEffort",
						"startTime": "2022-04-18T21:03:19Z",
					},
				},
			},
		}
		data.Items = append(data.Items, item)
	}
	return data
}

func newBenchmarkYAML() ([]byte, error) {
	return yaml.Marshal(newBenchmarkObject())
}

func BenchmarkMarshal(b *testing.B) {
	// Setup
	obj := newBenchmarkObject()

	// Record the number of bytes per operation
	result, err := Marshal(obj)
	if err != nil {
		b.Errorf("error marshaling YAML: %v", err)
	}
	b.SetBytes(int64(len(result)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if _, err := Marshal(obj); err != nil {
				b.Errorf("error marshaling YAML: %v", err)
			}
		}
	})
}

func BenchmarkUnmarshal(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}

	// Record the number of bytes per operation
	b.SetBytes(int64(len(yamlBytes)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			var result interface{}
			if err = Unmarshal(yamlBytes, &result); err != nil {
				b.Errorf("error unmarshaling YAML: %v", err)
			}
		}
	})
}

func BenchmarkUnmarshalStrict(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}

	// Record the number of bytes per operation
	b.SetBytes(int64(len(yamlBytes)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			var result interface{}
			if err = UnmarshalStrict(yamlBytes, &result); err != nil {
				b.Errorf("error unmarshaling YAML (Strict): %v", err)
			}
		}
	})
}

func BenchmarkJSONToYAML(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}
	jsonBytes, err := YAMLToJSON(yamlBytes)
	if err != nil {
		b.Fatalf("error initializing JSON: %v", err)
	}

	// Record the number of bytes per operation
	result, err := JSONToYAML(jsonBytes)
	if err != nil {
		b.Errorf("error converting JSON to YAML: %v", err)
	}
	b.SetBytes(int64(len(result)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if _, err := JSONToYAML(jsonBytes); err != nil {
				b.Errorf("error converting JSON to YAML: %v", err)
			}
		}
	})
}

func BenchmarkYAMLtoJSON(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}

	// Record the number of bytes per operation
	result, err := YAMLToJSON(yamlBytes)
	if err != nil {
		b.Errorf("error converting YAML to JSON: %v", err)
	}
	b.SetBytes(int64(len(result)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if _, err := YAMLToJSON(yamlBytes); err != nil {
				b.Errorf("error converting YAML to JSON: %v", err)
			}
		}
	})
}

func BenchmarkYAMLtoJSONStrict(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}

	// Record the number of bytes per operation
	result, err := YAMLToJSONStrict(yamlBytes)
	if err != nil {
		b.Errorf("error converting YAML to JSON (Strict): %v", err)
	}
	b.SetBytes(int64(len(result)))

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			if _, err := YAMLToJSONStrict(yamlBytes); err != nil {
				b.Errorf("error converting YAML to JSON (Strict): %v", err)
			}
		}
	})
}

func BenchmarkJSONObjectToYAMLObject(b *testing.B) {
	// Setup
	yamlBytes, err := newBenchmarkYAML()
	if err != nil {
		b.Fatalf("error initializing YAML: %v", err)
	}
	jsonBytes, err := YAMLToJSON(yamlBytes)
	if err != nil {
		b.Fatalf("error initializing JSON: %v", err)
	}
	var m map[string]interface{}
	err = json.Unmarshal(jsonBytes, &m)
	if err != nil {
		b.Fatalf("error initializing map: %v", err)
	}

	// Start the benchmark
	b.ResetTimer()
	b.ReportAllocs()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			JSONObjectToYAMLObject(m)
		}
	})
}
