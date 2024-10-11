#!/bin/bash

set -exu -o pipefail
[[ -f /VERSION ]] && cat /VERSION

cd github

export GOPATH="${HOME}/gopath"
pushd grpc-go/interop/xds/client
# Install a version of Go supported by gRPC for the new features, e.g.
# errors.Is()
gofilename=go1.21.0.linux-amd64.tar.gz
curl --retry 3 -O -L "https://go.dev/dl/${gofilename}"
sudo tar -C /usr/local -xf "${gofilename}"
sudo ln -s /usr/local/go/bin/go /usr/bin/go
# Retry go build on errors (e.g. go get connection errors), for at most 3 times
for i in 1 2 3; do go build && break || sleep 5; done
popd

git clone -b master --single-branch --depth=1 https://github.com/grpc/grpc.git

grpc/tools/run_tests/helper_scripts/prep_xds.sh

# Test cases "path_matching" and "header_matching" are not included in "all",
# because not all interop clients in all languages support these new tests.
#
# TODO: remove "path_matching" and "header_matching" from --test_case after
# they are added into "all".
GRPC_GO_LOG_VERBOSITY_LEVEL=99 GRPC_GO_LOG_SEVERITY_LEVEL=info \
  python3 grpc/tools/run_tests/run_xds_tests.py \
    --test_case="ping_pong,circuit_breaking" \
    --project_id=grpc-testing \
    --project_num=830293263384 \
    --source_image=projects/grpc-testing/global/images/xds-test-server-5 \
    --path_to_server_binary=/java_server/grpc-java/interop-testing/build/install/grpc-interop-testing/bin/xds-test-server \
    --gcp_suffix=$(date '+%s') \
    --verbose \
    ${XDS_V3_OPT-} \
    --client_cmd="grpc-go/interop/xds/client/client \
      --server=xds:///{server_uri} \
      --stats_port={stats_port} \
      --qps={qps} \
      {fail_on_failed_rpc} \
      {rpcs_to_send} \
      {metadata_to_send}"
