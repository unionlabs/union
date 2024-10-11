#!/bin/bash

# Copyright 2021 Google LLC.
# Use of this source code is governed by a BSD-style
# license that can be found in the LICENSE file.


# This script is used to generate the project configurations needed to
# end-to-end test Downscoping with Credential Access Boundaries in the Auth
# library. This script only needs to be run once.
#
# In order to run this script, you need to fill in the project_id and
# service_account_email variables. 
#
# If an argument is provided, the script will use the provided argument
# as the bucket name.  Otherwise, it will create a new bucket.
#
# This script needs to be run once. It will do the following:
# 1. Sets the current project to the one specified.
# 2. If no bucket name was provided, creates a GCS bucket in the specified project.
# 3. Gives the specified service account the objectAdmin role for this bucket.
# 4. Creates two text files to be uploaded to the created bucket.
# 5. Uploads both text files.
# 6. Prints out the identifiers (bucket ID, first object ID, second object ID)
#    to be used in the accompanying tests. 
# 7. Deletes the created text files in the current directory. 
# 
# The same service account used for this setup script should be used for
# the integration tests.
#
# It is safe to run the setup script again. A new bucket is created along with
# new objects. If run multiple times, it is advisable to delete
# unused buckets. 

suffix=""

function generate_random_string () {
  local valid_chars=abcdefghijklmnopqrstuvwxyz0123456789
  for i in {1..8} ; do
    suffix+="${valid_chars:RANDOM%${#valid_chars}:1}"
    done
}

generate_random_string

first_object="cab-first-"${suffix}.txt
second_object="cab-second-"${suffix}.txt

# Fill in.
project_id="dulcet-port-762"
service_account_email="kokoro@dulcet-port-762.iam.gserviceaccount.com"

gcloud config set project ${project_id}

if (( $# != 1 ))
then
	# Create the GCS bucket.
	bucket_id="cab-int-bucket-"${suffix}
	gsutil mb -b on -l us-east1 gs://${bucket_id}  
else
	bucket_id="$1"
fi

# Give the specified service account the objectAdmin role for this bucket.
gsutil iam ch serviceAccount:${service_account_email}:objectAdmin gs://${bucket_id}

# Create both objects.
echo "first" >> ${first_object}
echo "second" >> ${second_object}

# Upload the created objects to the bucket.
gsutil cp ${first_object} gs://${bucket_id}
gsutil cp ${second_object} gs://${bucket_id}

echo "Bucket ID: "${bucket_id}
echo "First object ID: "${first_object}
echo "Second object ID: "${second_object}

# Cleanup
rm ${first_object}
rm ${second_object}
