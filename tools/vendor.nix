{ ... }: {
  perSystem =
    { pkgs
    , self'
    , crane
    , rust
    , system
    , ensureAtRepositoryRoot
    , libwasmvmCargoToml_1_3_0
    , libwasmvmCargoToml_1_5_0
    , sqlxCliCargoToml
    , mkCi
    , ...
    }:
    let
      vendorDir = "tools/vendor/";
      vendorDirPath = ./vendor;

      srcWithVendoredSources = { name, originalSrc }:
        let
          configToml = ''
            [source.crates-io]
            replace-with = "vendored-sources"

            [source."git+https://github.com/CosmWasm/cosmwasm.git?rev=v1.3.0"]
            git = "https://github.com/CosmWasm/cosmwasm.git"
            rev = "v1.3.0"
            replace-with = "vendored-sources"

            [source."git+https://github.com/CosmWasm/cosmwasm.git?rev=v1.5.0"]
            git = "https://github.com/CosmWasm/cosmwasm.git"
            rev = "v1.5.0"
            replace-with = "vendored-sources"

            [source.vendored-sources]
            directory = "tools/vendor/"
          '';
        in
        pkgs.stdenv.mkDerivation {
          name = "${name}-vendored-sources-cargo-config-toml";
          src = originalSrc;
          buildInputs = [ pkgs.moreutils pkgs.jq ];
          buildPhase = ''
            cp -r . $out

            mkdir -p $out/${vendorDir}
          
            cp -r --no-preserve=mode ${vendorDirPath}/. $out/${vendorDir}/

            # FIXME: This is necessary due to git-lfs issues, likely not required once we go open source
            for file in $(find $out/${vendorDir} -type f -name ".cargo-checksum.json");
            do
              # -exec echo $'\u7b\u7d' > {} \;
              jq '.files = {}' $file | sponge $file;
            done

            mkdir -p $out/.cargo
            echo '${configToml}' >> $out/.cargo/config.toml
          '';
        };
    in
    {
      _module.args.srcWithVendoredSources = srcWithVendoredSources;

      packages.vendor-tools =
        let
          args = pkgs.lib.concatStringsSep
            " "
            (pkgs.lib.lists.imap0
              (i: tool:
                if i == 0
                then
                  "--manifest-path ${tool}"
                else
                  "--sync ${tool}"
              )
              [ libwasmvmCargoToml_1_3_0 libwasmvmCargoToml_1_5_0 sqlxCliCargoToml ]
            );
        in
        mkCi false (pkgs.writeShellApplication {
          name = "vendor-tools";
          text =
            ''
              ${ensureAtRepositoryRoot}

              cargo --version

              CARGO_NET_GIT_FETCH_WITH_CLI=true cargo vendor --locked ${args} ${vendorDir}
            '';
        });
    };
}
