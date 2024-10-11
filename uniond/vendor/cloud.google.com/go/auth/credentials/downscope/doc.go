// Copyright 2023 Google LLC
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

// Package downscope implements the ability to downscope, or restrict, the
// Identity and Access Management permissions that a short-lived Token
// can use. Please note that only Google Cloud Storage supports this feature.
// For complete documentation, see https://cloud.google.com/iam/docs/downscoping-short-lived-credentials
//
// To downscope permissions of a source credential, you need to define
// a Credential Access Boundary. Said Boundary specifies which resources
// the newly created credential can access, an upper bound on the permissions
// it has over those resources, and optionally attribute-based conditional
// access to the aforementioned resources. For more information on IAM
// Conditions, see https://cloud.google.com/iam/docs/conditions-overview.
//
// This functionality can be used to provide a third party with
// limited access to and permissions on resources held by the owner of the root
// credential or internally in conjunction with the principle of least privilege
// to ensure that internal services only hold the minimum necessary privileges
// for their function.
//
// For example, a token broker can be set up on a server in a private network.
// Various workloads (token consumers) in the same network will send
// authenticated requests to that broker for downscoped tokens to access or
// modify specific google cloud storage buckets. See the NewCredentials example
// for an example of how a token broker would use this package.
//
// The broker will use the functionality in this package to generate a
// downscoped token with the requested configuration, and then pass it back to
// the token consumer. These downscoped access tokens can then be used to access
// Google Cloud resources.
package downscope
