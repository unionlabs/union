{ ... }: {
  perSystem = { self', pkgs, rust, system, lib, dbg, inputs', goPkgs,  ensureAtRepositoryRoot,...}: {
    packages = let
      interchaintest = goPkgs.buildGoModule {
        pname = "interchaintest";
        version = "v8.0.0";
        src = builtins.fetchGit {
          name = "interchaintest";
          url = "git@github.com:unionlabs/interchaintest";
          rev = "67c5183e17094ecff6a75b81ce750f10cbd70f34";
          allRefs = true;
        };
        # src = inputs'.interchaintest;
        vendorHash = null;
        buildPhase = ''
          runHook preBuild
          runHook renameImports

          export GOWORK=off
          echo "LINENO: $LINENO"

          echo "pwd: $(pwd)"
          echo "ls: $(ls -a)"
          echo "ls nixbuildtop/go/bin: $(ls $NIX_BUILD_TOP/go/bin)"

          go clean -modcache

        	go test -c -o $out/bin/interchaintest ./cmd/interchaintest

          echo "ls bin: $(ls ./bin)"

          runHook postBuild
        '';
        doCheck = false;
      };
      imageReplaceString = "REPLACE_IMAGE_TAG";
      unionMatrix = {
        Relayers = [ "rly" ];
        ChainSets = [
          [
            {
              Name = "union";
              Type = "cosmos";
              ChainId = "union-devnet";
              Images = [
                {
                  Repository = "uniond";
                  Version = imageReplaceString;
                  UidGid = "1025:1025";
                }
              ];
              Bin = "uniond";
              Bech32Prefix = "union";
              Denom = "muno";
              GasPrices = "0.01muno";
              GasAdjustment = 1.3;
              TrustingPeriod = "504h";
              InitExtraArgs = "bn254";
            }
            {
              Name = "osmosis";
              Version = "latest";
            }
          ]
        ];
      };
      union_interchaintest_config = pkgs.linkFarm "union_interchaintest_config" [
        {
          name = "union_matrix.json";
          path = pkgs.writeText "union_matrix.json" (builtins.toJSON unionMatrix);
        }
      ];
    in
    {
      interchaintest-uniond-conformance = pkgs.writeShellApplication {
        name = "interchaintest-uniond-conformance";
        runtimeInputs = [ interchaintest self'.packages.uniond-release-image union_interchaintest_config ];
        text = ''
          set -eo pipefail
          ${ensureAtRepositoryRoot}

          docker_string=$(docker load < ${self'.packages.uniond-release-image})
          image_tag="''${docker_string##*':'}"

          cp ${union_interchaintest_config}/union_matrix.json .

          sed -i "s/${imageReplaceString}/$image_tag/g" union_matrix.json

          jq . < union_matrix.json

          echo "Running interchaintest conformance check for uniond..."

          ${interchaintest}/bin/interchaintest -matrix union_matrix.json
        '';
      };
    };
  };
}
