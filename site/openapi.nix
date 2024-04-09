{ ... }: {
  perSystem = { pkgs, self', openapi, ensureAtRepositoryRoot, mkCi, ... }: {
    packages =
      let
        generate-openapi-build-phase = pkgs.stdenv.mkDerivation {
          name = "generate-openapi-build-phase";
          pname = "generate-openapi-build-phase";
          src = ./.;
          buildInputs = [
            pkgs.yq
            pkgs.jq
          ];
          buildPhase = ''
            echo "Build Phase"
            mkdir $out

            yq <${openapi.ibcGoOpenApiYml} | jq --compact-output >"$out"/ibc_go_v2.json
            yq <${openapi.uniondOpenApiYml} | jq --compact-output >"$out"/union_rest_v2.json
            yq <${openapi.cometblsOpenApiYml} | jq --compact-output >"$out"/cometbls_v3.json
          '';
        };
      in
      {
        generate-openapi = mkCi false (pkgs.writeShellApplication {
          name = "generate-openapi";
          runtimeInputs = (with pkgs; [ yq jq tree curl ]);
          text = ''
            set -eou pipefail
            ${ensureAtRepositoryRoot}

            echo "Generating OpenAPI"

            mkdir -p /tmp/openapi/rpc
            mkdir -p /tmp/openapi/rest

            # convert ibc-go spec to v3
            curl --silent \
              --request 'POST' \
              --url 'https://converter.swagger.io/api/convert' \
              --header 'accept: application/json' \
              --header 'Content-Type: application/json' \
              --data-binary "@${generate-openapi-build-phase}/ibc_go_v2.json" \
              --output /tmp/ibc_go_v3.json
              
            # convert union-rest spec to v3
            curl --silent \
              --request 'POST' \
              --url 'https://converter.swagger.io/api/convert' \
              --header 'accept: application/json' \
              --header 'Content-Type: application/json' \
              --data-binary "@${generate-openapi-build-phase}/union_rest_v2.json" \
              --output /tmp/union_rest_v3.json

            # take `paths` from ibc-go v3 spec and merge with `paths` from union-rest v3 spec
            jq --slurpfile ibc_go_v3 \
              /tmp/ibc_go_v3.json '.paths += $ibc_go_v3[0].paths' /tmp/union_rest_v3.json \
              >/tmp/union_rest_v3_merged.json

            # take 'components.schemas' from ibc-go v3 spec and merge with 'components.schemas' from union-rest v3 spec
            jq \
              --slurpfile ibc_go_v3 /tmp/ibc_go_v3.json \
              '.components.schemas += $ibc_go_v3[0].components.schemas' /tmp/union_rest_v3_merged.json \
              >/tmp/union_rest_v3_merged_schemas.json

            ENDPOINTS=$(
              curl --silent \
                --request GET \
                --url 'https://union.build/api/endpoints.json' | jq
            )

            # replace the `servers` field in union_rest_v3_merged_schemas.json with `echo $ENDPOINTS | jq --compact-output '[.rest[] | {url: .}]'`
            REST_ENDPOINTS=$(echo "$ENDPOINTS" | jq --compact-output '[.rest[] | {url: .}]')
            jq \
              --argjson rest_endpoints "$REST_ENDPOINTS" '.servers = $rest_endpoints' \
              /tmp/union_rest_v3_merged_schemas.json >/tmp/union_rest_v3_merged_schemas_servers.json

            # finally, store the union_rest_v3_merged_schemas_servers.json in the correct location
            mv /tmp/union_rest_v3_merged_schemas_servers.json /tmp/openapi/rest/openapi.json

            # remove everything in `info` except `title` and `version`
            jq '.info = {title: .info.title, version: .info.version}' ${generate-openapi-build-phase}/cometbls_v3.json >/tmp/cometbls_v3_info.json

            # replace the `servers` field in cometbls_v3_info.json with `echo $ENDPOINTS | jq --compact-output '[.rpc[] | {url: .}]'`
            RPC_ENDPOINTS=$(echo "$ENDPOINTS" | jq --compact-output '[.rpc[] | {url: .}]')
            jq \
              --argjson rpc_endpoints "$RPC_ENDPOINTS" '.servers = $rpc_endpoints' \
              /tmp/cometbls_v3_info.json >/tmp/cometbls_v3_info_servers.json

            # finally, store the cometbls_v3_info_servers.json in the correct location
            mv /tmp/cometbls_v3_info_servers.json /tmp/openapi/rpc/openapi.json

            < /tmp/openapi/rest/openapi.json jq --compact-output . >site/src/content/openapi/rest/openapi.json
            < /tmp/openapi/rpc/openapi.json jq --compact-output . >site/src/content/openapi/rpc/openapi.json
          '';
        });
      };
    checks = { };
  };
}
