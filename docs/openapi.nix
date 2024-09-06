{ ... }: {
  perSystem = { pkgs, unstablePkgs, self', openapi, ensureAtRepositoryRoot, mkCi, biome, ... }: {
    packages =
      let
        generate-openapi-build-phase = pkgs.runCommand "generate-openapi-build"
          {
            buildInputs = [ biome ] ++ (with unstablePkgs; [ yq jq bun openapi-generator-cli ]);
            BIOME_JSON = ../biome.json;
            TS_OPENAPI_CLEANUP_SCRIPT = ./scripts/openapi-cleanup.ts;
          } ''
          echo "Build Phase"

          mkdir $out
          mkdir -p $out/openapi/rpc/
          mkdir -p $out/openapi/rest/
            
          yq <${openapi.cometblsOpenApiYml} | jq --compact-output >cometbls_rpc_v3.json && \
            #
            # remove everything in `info` except `title` and `version`
            jq '.info = {title: .info.title, version: .info.version}' cometbls_rpc_v3.json >/tmp/cometbls_v3_info.json && \
            #
            # remove all `servers` from the spec and replace with a single server `[{"url": "http://localhost:26657" }]`
            #
            jq '.servers = [{"url": "http://localhost:26657" }]' /tmp/cometbls_v3_info.json >/tmp/cometbls_v3_info_servers.json && \
            #
            # finally, store the cometbls_v3_info.json in the correct location
            #
            mv /tmp/cometbls_v3_info_servers.json "$out"/openapi/rpc/openapi.json

          #
          # convert ibc openapi spec to openapi v3
          #
          openapi-generator-cli generate \
            --generator-name openapi \
            --output ibc-go-out \
            --input-spec ${openapi.ibcGoOpenApiYml} 

          #
          # convert uniond openapi spec to openapi v3
          #
          openapi-generator-cli generate \
            --generator-name openapi \
            --output uniond-out \
            --input-spec ${openapi.uniondOpenApiYml}

          #
          # take `paths` from ibc-go v3 spec and merge with `paths` from union-rest v3 spec
          #
          jq --slurpfile ibc_go_v3 \
            ibc-go-out/openapi.json '.paths += $ibc_go_v3[0].paths' uniond-out/openapi.json \
            >/tmp/union_rest_v3_merged.json && \
            #
            # take 'components.schemas' from ibc-go v3 spec and merge with 'components.schemas' from union-rest v3 spec
            #
            jq \
              --slurpfile ibc_go_v3 ibc-go-out/openapi.json \
              '.components.schemas += $ibc_go_v3[0].components.schemas' /tmp/union_rest_v3_merged.json \
              >/tmp/union_rest_v3_merged_schemas.json && \
              #
              # remove unused `paths` and `components.schemas` from the merged spec
              #
              bun $TS_OPENAPI_CLEANUP_SCRIPT /tmp/union_rest_v3_merged_schemas.json > /tmp/union_rest_v3_cleaned.json && \
              #
              # finally, store the union_rest_v3_cleaned.json in the correct location
              #
              mv /tmp/union_rest_v3_cleaned.json "$out"/openapi/rest/openapi.json && \
              #
              # validate the generated RPC openapi specs
              #
              openapi-generator-cli validate \
                --input-spec "$out"/openapi/rpc/openapi.json \
                --recommend
            
          #
          # validate the generated REST openapi specs
          #
          openapi-generator-cli validate \
            --input-spec "$out"/openapi/rest/openapi.json \
            --recommend

          #
          # format the result
          #
          cp $BIOME_JSON ./biome.json
          biome format $out --config-path . --error-on-warnings --log-level="info" --diagnostic-level="info" --write
        '';
      in
      {
        generate-openapi = mkCi false (pkgs.writeShellApplication {
          name = "generate-openapi";
          text = ''
            set -eou pipefail
            ${ensureAtRepositoryRoot}

            echo "Copying generated OpenAPI to site"

            cat ${generate-openapi-build-phase}/openapi/rpc/openapi.json > docs/src/content/openapi/rpc/openapi.json
            cat ${generate-openapi-build-phase}/openapi/rest/openapi.json > docs/src/content/openapi/rest/openapi.json
          '';
        });
      };
    checks = { };
  };
}
