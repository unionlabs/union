#!/bin/bash
# Copyright 2021 Google LLC.
# Use of this source code is governed by a BSD-style
# license that can be found in the LICENSE file.

# This file is a mostly common setup file to ensure all BYOID integration tests
# are set up in a consistent fashion.
# It assumes that the current user has the relevant permissions to run each of
# the commands listed.

suffix=""

function generate_random_string () {
  local valid_chars=abcdefghijklmnopqrstuvwxyz0123456789
  for i in {1..8} ; do
    suffix+="${valid_chars:RANDOM%${#valid_chars}:1}"
    done
}

generate_random_string

pool_id="pool-"$suffix
oidc_provider_id="oidc-"$suffix
aws_provider_id="aws-"$suffix

# Fill in.
project_id=$GCLOUD_TESTS_GOLANG_PROJECT_ID
project_number=$GCLOUD_TESTS_GOLANG_PROJECT_NUMBER
aws_account_id=$GCLOUD_TESTS_GOLANG_AWS_ACCOUNT_ID
aws_role_name=$GCLOUD_TESTS_GOLANG_AWS_ROLE_NAME
service_account_email=$GCLOUD_TESTS_GOLANG_SERVICE_ACCOUNT_EMAIL
sub=$GCLOUD_TESTS_GOLANG_SERVICE_ACCOUNT_CLIENT_ID

oidc_aud="//iam.googleapis.com/projects/$project_number/locations/global/workloadIdentityPools/$pool_id/providers/$oidc_provider_id"
aws_aud="//iam.googleapis.com/projects/$project_number/locations/global/workloadIdentityPools/$pool_id/providers/$aws_provider_id"

gcloud config set project $project_id

# Create the Workload Identity Pool.
gcloud beta iam workload-identity-pools create $pool_id \
    --location="global" \
    --description="Test pool" \
    --display-name="Test pool for Go"

# Create the OIDC Provider.
gcloud beta iam workload-identity-pools providers create-oidc $oidc_provider_id \
    --workload-identity-pool=$pool_id \
    --issuer-uri="https://accounts.google.com" \
    --location="global" \
    --attribute-mapping="google.subject=assertion.sub"

# Create the AWS Provider.
gcloud beta iam workload-identity-pools providers create-aws $aws_provider_id \
    --workload-identity-pool=$pool_id \
    --account-id=$aws_account_id \
    --location="global"

# Give permission to impersonate the service account.
gcloud iam service-accounts add-iam-policy-binding $service_account_email \
--role roles/iam.workloadIdentityUser \
--member "principal://iam.googleapis.com/projects/$project_number/locations/global/workloadIdentityPools/$pool_id/subject/$sub"

gcloud iam service-accounts add-iam-policy-binding $service_account_email \
  --role roles/iam.workloadIdentityUser \
  --member "principalSet://iam.googleapis.com/projects/$project_number/locations/global/workloadIdentityPools/$pool_id/attribute.aws_role/arn:aws:sts::$aws_account_id:assumed-role/$aws_role_name"

echo "OIDC audience: "$oidc_aud
echo "AWS audience: "$aws_aud
