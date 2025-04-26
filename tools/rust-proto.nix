{ inputs, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      proto,
      crane,
      system,
      config,
      ensureAtRepositoryRoot,
      mkCi,
      ...
    }:
    let
      protoc-gen-tonic = crane.lib.buildPackage {
        pname = "protoc-gen-tonic";
        version = "0.0.1";
        doCheck = false;
        src = builtins.fetchGit {
          url = "https://github.com/neoeinstein/protoc-gen-prost";
          rev = "0548ae244f5780cdf0790ebf48b497a5df1acfc4";
        };
      };

      feemarket-repo = pkgs.fetchFromGitHub {
        owner = "skip-mev";
        repo = "feemarket";
        rev = "v1.1.1";
        sha256 = "sha256-MDrwJhzDKcPXbExViwYgoKeVhNB2CXkqj+iq8kUb2i8=";
      };

      cargo_toml =
        { name }:
        let
          toml = {
            package = {
              inherit name;
              version = "0.0.0";
              edition = "2021";
            };
            lib = {
              doctest = false;
            };
            dependencies = {
              prost = {
                workspace = true;
                features = [ "prost-derive" ];
              };
              serde = {
                workspace = true;
                features = [ "derive" ];
                optional = true;
              };
              serde-utils = {
                workspace = true;
              };
            };
            features = {
              default = [
                "proto_full"
                "std"
              ];
              std = [
                "prost/std"
                "serde/std"
              ];
              # nix attrsets don't preserve order, use this to replace with the insertion point (see command below)
              PROTOC_INSERTION_POINT = 1;
            };
            lints = {
              workspace = true;
            };
          };
        in
        pkgs.runCommand "${name}-cargo_toml" { } ''
          cargotoml='${builtins.toJSON toml}'
          echo "cargo toml: $cargotoml"
          echo "$cargotoml" | ${pkgs.lib.meta.getExe pkgs.yj} -jt | sed 's/^PROTOC_INSERTION_POINT = 1$/## @@protoc_insertion_point(features)/' > $out
        '';

      all-protos-to-build = rec {
        babylon = rec {
          src = "${inputs.babylon}/proto";
          proto-deps = [
            "${proto.gogoproto}/protobuf"
            src
            # "${proto.cosmosproto}/proto"
            # cosmos-sdk.src
            # google.src
          ];
        };
        osmosis = rec {
          src = "${inputs.osmosis}/proto";
          proto-deps = [
            "${proto.gogoproto}/protobuf"
            src
            # "${proto.cosmosproto}/proto"
            # cosmos-sdk.src
            # google.src
          ];
        };
        wasmd = rec {
          src = "${proto.wasmd}/proto";
          proto-deps = [
            src
            google.src
            "${proto.cosmossdk}/proto"
            ics23.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
        };
        cometbls = rec {
          src = "${proto.cometbls}/proto";
          proto-deps = [
            src
            google.src
            "${proto.gogoproto}"
          ];
          # inject https://github.com/cometbft/cometbft/blob/main/proto/cometbft/crypto/v1/keys.proto#L17
          # yes prost is incredibly cursed. we have to annotate the field that uses the generated oneof `Sum` type with the additional enum tag for some reason? no clue why this information has to be duplicated.
          # ask me how much time i wasted figuring this out
          fixup-script = ''
            sed -i 's/#\[prost(oneof = "public_key::Sum", tags = "1, 2")\]/#\[prost(oneof = "public_key::Sum", tags = "1, 2, 3, 99")\]/' "./src/tendermint.crypto.rs"
            sed -i 's/Secp256k1(::prost::alloc::vec::Vec<u8>),/Secp256k1(::prost::alloc::vec::Vec<u8>),#\[prost(bytes, tag = "3")\]Bn254(::prost::alloc::vec::Vec<u8>),#\[prost(bytes, tag = "99")\]Bls12381(::prost::alloc::vec::Vec<u8>),/' "./src/tendermint.crypto.rs"
          '';
        };

        uniond = rec {
          src = "${proto.uniond}";
          proto-deps = [
            src
            # "${proto.cosmossdk}/x/bank/proto"
            # "${proto.cosmossdk}/x/staking/proto"
          ];
        };
        galoisd = rec {
          src = "${proto.galoisd}";
          proto-deps = [
            src
          ];
        };
        google = rec {
          src = "${proto.gogoproto}/protobuf";
          proto-deps = [
            src
          ];
          additional-filter = "-path '*google/protobuf/*.proto'";
        };
        ibc-proto = rec {
          src = "${proto.ibc-go}/proto";
          proto-deps = [
            src
            google.src
            "${proto.cosmossdk}/proto"
            ics23.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
        };
        ics23 = rec {
          src = "${proto.ics23}/proto";
          proto-deps = [
            src
          ];
        };
        interchain-security = rec {
          src = "${proto.interchain-security}/proto";
          proto-deps = [
            src
            ibc-proto.src
            google.src
            cometbls.src
            # cosmos-sdk-evidence.src
          ];
        };
        cosmos-sdk = {
          src = "${proto.cosmossdk}/proto";
          proto-deps = [
            "${proto.cosmossdk}/proto"
            google.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
          # {msg,query}.proto have to be excluded since they only contain extensions, causing the generator to not create a file
          # SEE: https://github.com/neoeinstein/protoc-gen-prost/issues/61
          additional-filter = "-not -path '*cosmos/msg/textual/v1/textual.proto' -not -path '*cosmos/msg/v1/msg.proto' -not -path '*cosmos/query/v1/query.proto' -and -not -path '*/proto/tendermint/*'";
          fixup-script = ''
            sed -i 's/pub struct Validators/pub struct ValidatorsList/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/impl ::prost::Name for Validators/impl ::prost::Name for ValidatorsList/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/AllowList(Validators)/AllowList(ValidatorsList)/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/DenyList(Validators)/DenyList(ValidatorsList)/' "./src/cosmos.staking.v1beta1.rs"
          '';
        };
        # NOTE: Needed for v0.52+
        # cosmos-sdk-bank = {
        #   src = "${proto.cosmossdk}/x/bank/proto";
        #   proto-deps = [ ];
        # };
        # cosmos-sdk-staking = {
        #   src = "${proto.cosmossdk}/x/staking/proto";
        #   proto-deps = [ ];
        # };
        # cosmos-sdk-evidence = {
        #   src = "${proto.cosmossdk}/x/evidence/proto";
        #   proto-deps = [ ];
        # };
        feemarket = rec {
          src = "${feemarket-repo}/proto";
          proto-deps = [
            src
          ];
        };
      };

      fold-opts =
        attrs:
        with pkgs.lib;
        escapeShellArg (
          concatStringsSep "," (
            flatten (
              foldlAttrs (
                acc: opt-name: opt-value:
                acc
                ++ (
                  # protoc splits on commas so we have to escape any in the attribute values
                  foldlAttrs (
                    acc: name: values:
                    acc ++ (map (attr: "${opt-name}=${name}=${escape [ "," ] attr}") values)
                  ) [ ] opt-value
                )
              ) [ ] attrs
            )
          )
        );

      prost-opts =
        let
          serde = ''#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]'';
          serde_default = ''#[cfg_attr(feature = "serde", serde(default))]'';
          serde_base64 = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]'';
        in
        {
          type_attribute = {
            ".cosmos.ics23.v1" = [ serde ];
            ".cosmos.ics23.v1.InnerOp" = [ ];
            ".cosmos.ics23.v1.InnerSpec" = [ ];
            ".cosmos.ics23.v1.LeafOp" = [ ];
            ".cosmos.ics23.v1.ProofSpec" = [ ];
          };

          field_attribute = {
            ".cosmos.ics23.v1.LeafOp.hash" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prehash_key" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prehash_value" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.length" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prefix" = [
              serde_default
              serde_base64
            ];

            ".cosmos.ics23.v1.InnerOp.prefix" = [
              serde_base64
              serde_default
            ];
            ".cosmos.ics23.v1.InnerOp.suffix" = [
              serde_base64
              serde_default
            ];

            ".cosmos.ics23.v1.ProofSpec.max_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.min_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.prehash_key_before_comparison" = [ serde_default ];

            ".cosmos.ics23.v1.InnerSpec.empty_child" = [
              serde_default
              serde_base64
            ];

            ".cosmos.ics23.v1.ExistenceProof.key" = [ serde_base64 ];
            ".cosmos.ics23.v1.ExistenceProof.value" = [ serde_base64 ];
            ".cosmos.ics23.v1.ExistenceProof.path" = [ serde_default ];
            ".cosmos.ics23.v1.ExistenceProof.leaf" = [ serde_default ];

            ".cosmos.ics23.v1.NonExistenceProof.value" = [ serde_base64 ];
          };

          enum_attribute = { };
        };

      proto-inputs =
        name:
        {
          src,
          additional-filter ? null,
          ...
        }:
        let
          af = if additional-filter != null then "-and " + additional-filter else "";
        in
        pkgs.runCommand "${name}-inputs" { } ''
          find ${src} -type f -name '*.proto' ${af} > $out
        '';

      fixup-scripts =
        with pkgs.lib;
        concatStringsSep "\n\n" (
          flatten (
            mapAttrsToList (
              name:
              {
                fixup-script ? null,
                ...
              }:
              optionalString (fixup-script != null) ''
                echo "[FIXUP] ${name}"

                ${fixup-script}
              ''
            ) all-protos-to-build
          )
        );

      includes =
        with pkgs.lib;
        concatStringsSep " " (
          flatten (
            mapAttrsToList (_: { proto-deps, ... }: map (src: ''-I"${src}"'') proto-deps) all-protos-to-build
          )
        );

      rust-proto = pkgs.stdenv.mkDerivation {
        name = "rust-proto";
        pname = "rust-proto";
        src = pkgs.linkFarm "rust-proto-srcs" (
          pkgs.lib.mapAttrsToList (
            name:
            { src, ... }:
            {
              name = name + "-protos";
              path = src;
            }
          ) all-protos-to-build
        );
        buildInputs = [
          pkgs.protobuf
          protoc-gen-tonic
          config.treefmt.build.programs.rustfmt
          pkgs.taplo
        ] ++ (if pkgs.stdenv.isDarwin then [ pkgs.libiconv ] else [ ]);
        buildPhase = ''
          mkdir $out

          readarray -t protos < <(cat ${pkgs.lib.concatStringsSep " " (pkgs.lib.mapAttrsToList proto-inputs all-protos-to-build)})

          for file in "''${protos[@]}"; do
            echo "[FOUND] $file"
          done

          mkdir ./src

          protoc "''${protos[@]}" \
            --prost_opt=compile_well_known_types \
            --prost_out=./src \
            --prost_opt=enable_type_names=true,compile_well_known_types=true,${fold-opts prost-opts} \
            --prost-crate_out=. \
            --prost-crate_opt=package_separator="+",gen_crate=${cargo_toml { name = "protos"; }} \
            ${includes}

          ${fixup-scripts}

          # prepend clippy and rustdoc allow to root lib.rs file
          echo -e "#![allow(clippy::all, rustdoc::all)]\n$(cat ./src/lib.rs)" > ./src/lib.rs

          # format generated files
          # echo 'formatter = { rust = { options = []}}' > treefmt.toml
          # touch flake.nix # treefmt looks for this file to find the project root
          # treefmt -C . --no-cache --config-file="treefmt.toml"

          # normalize comments in generated code
          for i in $(find . -name "*.rs" -type f); do
            echo "[FORMAT] $i"
            rustfmt --config-path=${../rustfmt.toml} --config normalize_comments=true --edition "2021" "$i"
          done

          taplo format --config=${../taplo.toml} ./Cargo.toml

          cp -r ./src $out/
          cp -r ./Cargo.toml $out/
        '';
      };
    in
    {
      packages.rust-proto = mkCi false rust-proto;

      packages.generate-rust-proto = mkCi false (
        pkgs.writeShellApplication {
          name = "generate-rust-proto";
          runtimeInputs = [
            rust-proto
            pkgs.rsync
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            outdir="generated/rust/protos/"

            mkdir -p "$outdir"

            rsync -rL --chmod=ugo=rwX --delete ${rust-proto}/ $outdir

            echo "Generation successful!"
          '';
        }
      );

      checks = {
        rust-proto-check = mkCi false (
          pkgs.stdenv.mkDerivation {
            name = "rust-proto-is-committed";
            description = "check that rust protos in git repo are the same as those that are generated in rust-proto derivation";
            src = ../.;
            buildInputs = [ pkgs.git ];
            doCheck = true;
            checkPhase = ''
              rust_protos_in_git_repo=./generated/rust/protos
              rust_protos_in_derivation=${self'.packages.rust-proto}
              git --no-pager diff --exit-code --no-index $rust_protos_in_git_repo $rust_protos_in_derivation
              touch $out
            '';
          }
        );
      };
    };
}
