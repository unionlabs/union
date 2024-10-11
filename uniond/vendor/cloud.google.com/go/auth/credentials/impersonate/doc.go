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

// Package impersonate is used to impersonate Google Credentials. If you need
// to impersonate some credentials to use with a client library see
// [NewCredentials]. If instead you would like to create an Open
// Connect ID token using impersonation see [NewIDTokenCredentials].
//
// # Required IAM roles
//
// In order to impersonate a service account the base service account must have
// the Service Account Token Creator role, roles/iam.serviceAccountTokenCreator,
// on the service account being impersonated. See
// https://cloud.google.com/iam/docs/understanding-service-accounts.
//
// Optionally, delegates can be used during impersonation if the base service
// account lacks the token creator role on the target. When using delegates,
// each service account must be granted roles/iam.serviceAccountTokenCreator
// on the next service account in the delgation chain.
//
// For example, if a base service account of SA1 is trying to impersonate target
// service account SA2 while using delegate service accounts DSA1 and DSA2,
// the following must be true:
//
//  1. Base service account SA1 has roles/iam.serviceAccountTokenCreator on
//     DSA1.
//  2. DSA1 has roles/iam.serviceAccountTokenCreator on DSA2.
//  3. DSA2 has roles/iam.serviceAccountTokenCreator on target SA2.
//
// If the base credential is an authorized user and not a service account, or if
// the option WithQuotaProject is set, the target service account must have a
// role that grants the serviceusage.services.use permission such as
// roles/serviceusage.serviceUsageConsumer.
package impersonate
