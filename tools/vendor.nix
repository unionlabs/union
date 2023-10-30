{ ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, libwasmvmCargoToml, sqlxCliCargoToml, ... }:
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

            [source.vendored-sources]
            directory = "tools/vendor/"
          '';
        in
        pkgs.stdenv.mkDerivation {
          name = "${name}-vendored-sources-cargo-config-toml";
          src = originalSrc;
          buildPhase = ''
            cp -r . $out

            mkdir -p $out/${vendorDir}
          
            cp -r --no-preserve=mode ${vendorDirPath}/. $out/${vendorDir}/

            diff -r $out/${vendorDir} ${vendorDirPath}

            mkdir -p $out/.cargo
            echo '${configToml}' >> $out/.cargo/config.toml
          '';
        };
    in
    {
      _module.args.srcWithVendoredSources = srcWithVendoredSources;

      packages.vendor-tools =
        let
          args = pkgs.lib.concatStringsSep " " (pkgs.lib.lists.imap0 (i: tool: if i == 0 then "--manifest-path ${tool}" else "--sync ${tool}") [ libwasmvmCargoToml sqlxCliCargoToml ]);
        in
        pkgs.writeShellApplication {
          name = "vendor-tools";
          text =
            ''
              ${ensureAtRepositoryRoot}

              cargo --version

              cargo vendor --locked ${args} ${vendorDir}
            '';
        };
    };
}
