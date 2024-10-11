// Copyright 2024 Google LLC
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

// Package externalaccount provides support for creating workload identity
// federation and workforce identity federation token providers that can be used
// to access Google Cloud resources from external identity providers.
//
// # Workload Identity Federation
//
// Using workload identity federation, your application can access Google Cloud
// resources from Amazon Web Services (AWS), Microsoft Azure or any identity
// provider that supports OpenID Connect (OIDC) or SAML 2.0.
// Traditionally, applications running outside Google Cloud have used service
// account keys to access Google Cloud resources. Using identity federation,
// you can allow your workload to impersonate a service account.
// This lets you access Google Cloud resources directly, eliminating the
// maintenance and security burden associated with service account keys.
//
// Follow the detailed instructions on how to configure Workload Identity
// Federation in various platforms:
//
// - Amazon Web Services (AWS): https://cloud.google.com/iam/docs/workload-identity-federation-with-other-clouds#aws
// - Microsoft Azure: https://cloud.google.com/iam/docs/workload-identity-federation-with-other-clouds#azure
// - OIDC identity provider: https://cloud.google.com/iam/docs/workload-identity-federation-with-other-providers#oidc
// - SAML 2.0 identity provider: https://cloud.google.com/iam/docs/workload-identity-federation-with-other-providers#saml
//
// For OIDC and SAML providers, the library can retrieve tokens in fours ways:
// from a local file location (file-sourced credentials), from a server
// (URL-sourced credentials), from a local executable (executable-sourced
// credentials), or from a user defined function that returns an OIDC or SAML token.
// For file-sourced credentials, a background process needs to be continuously
// refreshing the file location with a new OIDC/SAML token prior to expiration.
// For tokens with one hour lifetimes, the token needs to be updated in the file
// every hour. The token can be stored directly as plain text or in JSON format.
// For URL-sourced credentials, a local server needs to host a GET endpoint to
// return the OIDC/SAML token. The response can be in plain text or JSON.
// Additional required request headers can also be specified.
// For executable-sourced credentials, an application needs to be available to
// output the OIDC/SAML token and other information in a JSON format.
// For more information on how these work (and how to implement
// executable-sourced credentials), please check out:
// https://cloud.google.com/iam/docs/workload-identity-federation-with-other-providers#create_a_credential_configuration
//
// To use a custom function to supply the token, define a struct that implements
// the [SubjectTokenProvider] interface for OIDC/SAML providers, or one that
// implements [AwsSecurityCredentialsProvider] for AWS providers. This can then
// be used when building a [Options].The [cloud.google.com/go/auth.Credentials]
// created from the options using [NewCredentials] can then be used to access
// Google Cloud resources. For instance, you can create a new client from the
// [cloud.google.com/go/storage] package and pass in
// option.WithTokenProvider(yourTokenProvider))
//
// # Workforce Identity Federation
//
// Workforce identity federation lets you use an external identity provider
// (IdP) to authenticate and authorize a workforce—a group of users, such as
// employees, partners, and contractors—using IAM, so that the users can access
// Google Cloud services. Workforce identity federation extends Google Cloud's
// identity capabilities to support syncless, attribute-based single sign on.
//
// With workforce identity federation, your workforce can access Google Cloud resources
// using an external identity provider (IdP) that supports OpenID Connect (OIDC) or
// SAML 2.0 such as Azure Active Directory (Azure AD), Active Directory Federation
// Services (AD FS), Okta, and others.
//
// Follow the detailed instructions on how to configure Workload Identity Federation
// in various platforms:
//
//   - [Amazon Web Services (AWS)](https://cloud.google.com/iam/docs/workload-identity-federation-with-other-clouds#aws)
//   - [Azure AD](https://cloud.google.com/iam/docs/workforce-sign-in-azure-ad)
//   - [Okta](https://cloud.google.com/iam/docs/workforce-sign-in-okta)
//   - [OIDC identity provider](https://cloud.google.com/iam/docs/configuring-workforce-identity-federation#oidc)
//   - [SAML 2.0 identity provider](https://cloud.google.com/iam/docs/configuring-workforce-identity-federation#saml)
//
// For workforce identity federation, the library can retrieve tokens in three ways:
// from a local file location (file-sourced credentials), from a server
// (URL-sourced credentials), or from a local executable (executable-sourced
// credentials).
// For file-sourced credentials, a background process needs to be continuously
// refreshing the file location with a new OIDC/SAML token prior to expiration.
// For tokens with one hour lifetimes, the token needs to be updated in the file
// every hour. The token can be stored directly as plain text or in JSON format.
// For URL-sourced credentials, a local server needs to host a GET endpoint to
// return the OIDC/SAML token. The response can be in plain text or JSON.
// Additional required request headers can also be specified.
// For executable-sourced credentials, an application needs to be available to
// output the OIDC/SAML token and other information in a JSON format.
// For more information on how these work (and how to implement
// executable-sourced credentials), please check out:
// https://cloud.google.com/iam/docs/workforce-obtaining-short-lived-credentials#generate_a_configuration_file_for_non-interactive_sign-in
//
// # Security considerations
//
// Note that this library does not perform any validation on the token_url,
// token_info_url, or service_account_impersonation_url fields of the credential
// configuration. It is not recommended to use a credential configuration that
// you did not generate with the gcloud CLI unless you verify that the URL
// fields point to a googleapis.com domain.
package externalaccount
