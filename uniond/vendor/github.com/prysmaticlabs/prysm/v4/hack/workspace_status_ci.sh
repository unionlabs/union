#!/bin/bash

repo_url=$(git config --get remote.origin.url)
echo "REPO_URL $repo_url"

commit_sha=$(git rev-parse HEAD)
echo "COMMIT_SHA $commit_sha"

echo "GIT_BRANCH $git_branch"

git_tree_status=$(git diff-index --quiet HEAD -- && echo 'Clean' || echo 'Modified')
echo "GIT_TREE_STATUS $git_tree_status"

# Note: the "STABLE_" suffix causes these to be part of the "stable" workspace
# status, which may trigger rebuilds of certain targets if these values change
# and you're building with the "--stamp" flag.
latest_version_tag=$(./hack/latest_version_tag.sh)
echo "STABLE_VERSION_TAG $latest_version_tag"
echo "STABLE_COMMIT_SHA $commit_sha"
echo "STABLE_GIT_COMMIT $commit_sha"
echo "STABLE_GIT_TAG $latest_version_tag"

echo DOCKER_TAG "$(git rev-parse --abbrev-ref HEAD)-$(git rev-parse --short=6 HEAD)"                                                                                                                         
echo DATE "$(date --rfc-3339=seconds --utc)"
echo DATE_UNIX "$(date --utc +%s)"
