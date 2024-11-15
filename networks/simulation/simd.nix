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
        simd = goPkgs.pkgsStatic.buildGo121Module (
          {
            name = "simd";
            src = pkgs.fetchFromGitHub {
              owner = "unionlabs";
              repo = "wasmd";
              rev = "5d478efa64b8f7557fd9e634ca25ad61708fc2b9";
              sha256 = "sha256-R5HPy2obHrLV0g8/1aAGAmLbWq6l5bgra6yX7l0ac4A=";
            };
            vendorHash = "sha256-rhuYWhaTtrHCeO9l4uiP7L2OmWkCPtMHXBqS7TRzM4s=";
            subPackages = [ "./cmd/wasmd" ];
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "wasmd";
          }
          // (
            let
              libwasmvm = self'.packages.libwasmvm-2_1_2;
              # libwasmvm = pkgs.stdenv.mkDerivation {
              #   name = "libwasmvm";
              #   src = pkgs.fetchurl {
              #     url = "https://github.com/CosmWasm/wasmvm/releases/download/v2.2.0-rc.2/libwasmvm_muslc.x86_64.a";
              #     hash = "sha256-LEl7UkbHIXpwxEfFARfH+wmQnsI+bkFRpN4+XynbgTQ=";
              #   };
              #   dontUnpack = true;
              #   buildPhase = ''
              #     mkdir -p $out/lib/
              #     cp $src $out/lib/libwasmvm.x86_64.a
              #   '';
              # };
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  goPkgs.musl
                  libwasmvm
                ];
                tags = [
                  "muslc"
                  "netgo"
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -static -L${goPkgs.musl}/lib -L${libwasmvm}/lib'"
                  "-X github.com/cosmos/cosmos-sdk/version.Name=wasmd"
                  "-X github.com/cosmos/cosmos-sdk/version.AppName=wasmd"
            		  "-X github.com/CosmWasm/wasmd/app.Bech32Prefix=wasm"
            		  "-X github.com/cosmos/cosmos-sdk/version.Version=v0.53.0"
            		  "-X github.com/cosmos/cosmos-sdk/version.BuildTags=muslc,netgo"
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
