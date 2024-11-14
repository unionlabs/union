_: {
  perSystem =
    {
      pkgs,
      goPkgs,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      dbg,
      ...
    }:
    {
      packages = {
        simd = goPkgs.pkgsStatic.buildGo123Module (
          {
            name = "simd";
            # src = builtins.fetchGit {
            #   url = "git@github.com:unionlabs/wasmd.git";
            #   rev = "a2e7048bbdd43206c69fb9353c6aff219aecefda";
            # };
            src = pkgs.fetchFromGitHub {
              owner = "cosmwasm";
              repo = "wasmd";
              rev = "4806a6e0607dabfef4f6e967919d50f313260496";
              sha256 = "sha256-WakFPkqpsTvA0Pr0Fuumxr2ZfSpaH9Q9xlCF9Q8hx14=";
            };
            vendorHash = "sha256-yPsi96gHG/ik7fssACHInjWNn52Ttda6cjvhE4aQbwM=";
            subPackages = [ "./cmd/wasmd" ];
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "wasmd";
            # CGO_ENABLED = 0;
          }
          // (
            let
              # libwasmvm = self'.packages.libwasmvm-2_2_0;
              libwasmvm = pkgs.stdenv.mkDerivation {
                name = "libwasmvm";
                src = pkgs.fetchurl {
                  url = "https://github.com/CosmWasm/wasmvm/releases/download/v2.2.0-rc.2/libwasmvm_muslc.aarch64.a";
                  hash = "sha256-evgOt+edgnieyg1VEqh9wg6WGCWQ/oiuX9AVPjHAl8k=";
                };
                dontUnpack = true;
                buildPhase = ''
                  mkdir -p $out/lib/
                  cp $src $out/lib/libwasmvm.aarch64.a
                '';
              };
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  goPkgs.musl
                  libwasmvm
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-z noexecstack -static -L${pkgs.musl}/lib -L${dbg libwasmvm}/lib'"
                ];
              }
            # else if pkgs.stdenv.isDarwin then {
            #   # Dynamically link if we're on darwin by wrapping the program
            #   # such that the DYLD_LIBRARY_PATH includes libwasmvm
            #   buildInputs = [ pkgs.makeWrapper libwasmvm ];
            #   postFixup = ''
            #     wrapProgram $out/bin/wasmd \
            #     --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
            #   '';
            # } else
            else
              { }
          )
        );

        simd-image = pkgs.dockerTools.buildImage {
          name = "simd";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils-full
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.simd) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };
}
