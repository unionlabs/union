{ ... }: {
  perSystem = { pkgs, ... }:
    {
      packages = {
        hasura-cli =
          let
            cli-ext-hashes = {
              linux-amd64 = "sha256-4XDLKIO/nT6LVaM6aSBpvKOMGhW5ca04iA1PMGBTWI8=";
              linux-arm64 = "sha256-GJ9Xtx1g0nIbirS9SOb/B8AeNhAMq6PDwx2HMiqXW9Q=";
              darwin-amd64 = "sha256-9qoW9SlGVafNP/t8xh3JMkpwWZ4BDR0QuEJO8g2sLHg=";
              darwin-arm64 = "sha256-wwWk/mOdnCeIan18Mj6qmwbmBUmtA8eqtR4g0UHrNBo=";
            };
            cli-ext = pkgs.stdenv.mkDerivation rec {
              pname = "hasura-cli-ext";
              version = "2.4.0-beta.3";
              src = pkgs.fetchurl {
                url = "https://graphql-engine-cdn.hasura.io/cli-ext/releases/versioned/v${version}/cli-ext-${pkgs.go.GOOS}-${pkgs.go.GOARCH}";
                sha256 = cli-ext-hashes."${pkgs.go.GOOS}-${pkgs.go.GOARCH}";
              };
              dontUnpack = true;
              phases = [ "installPhase" "fixupPhase" ];
              installPhase = ''
                mkdir -p $out/bin
                cp $src $out/bin/cli-ext
                chmod +x $out/bin/cli-ext
              '';
              preFixup =
                let
                  libPath = pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc ];
                in
                ''
                  orig_size=$(stat --printf=%s $out/bin/cli-ext)

                  patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" $out/bin/cli-ext
                  patchelf --set-rpath ${libPath} $out/bin/cli-ext
                  new_size=$(stat --printf=%s $out/bin/cli-ext)

                  ###### zeit-pkg fixing starts here.
                  # we're replacing plaintext js code that looks like
                  # PAYLOAD_POSITION = '1234                  ' | 0
                  # [...]
                  # PRELUDE_POSITION = '1234                  ' | 0
                  # ^-----20-chars-----^^------22-chars------^
                  # ^-- grep points here
                  #
                  # var_* are as described above
                  # shift_by seems to be safe so long as all patchelf adjustments occur
                  # before any locations pointed to by hardcoded offsets

                  var_skip=20
                  var_select=22
                  shift_by=$(expr $new_size - $orig_size)

                  function fix_offset {
                    # $1 = name of variable to adjust
                    location=$(grep -obUam1 "$1" $out/bin/cli-ext | cut -d: -f1)
                    location=$(expr $location + $var_skip)

                    value=$(dd if=$out/bin/cli-ext iflag=count_bytes,skip_bytes skip=$location \
                               bs=1 count=$var_select status=none)
                    value=$(expr $shift_by + $value)

                    echo -n $value | dd of=$out/bin/cli-ext bs=1 seek=$location conv=notrunc
                  }

                  fix_offset PAYLOAD_POSITION
                  fix_offset PRELUDE_POSITION
                '';
              dontStrip = true;
            };
          in
          pkgs.symlinkJoin {
            name = "hasura";
            paths = [ pkgs.hasura-cli ];
            buildInputs = [ pkgs.makeWrapper ];
            postBuild = ''
              wrapProgram $out/bin/hasura \
                --add-flags "--cli-ext-path ${cli-ext}/bin/cli-ext"
            '';
          };
      };
    };
}
