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
              owner = "cosmwasm";
              repo = "wasmd";
              rev = "37aedfdc5fe917b91347d0cc49c8ba0067f0d514";
              sha256 = "sha256-7Mzt5QcCoEs4qEF20/8YuZy538vdqywc2rL1ifdmgtU=";
            };
            vendorHash = "sha256-rhuYWhaTtrHCeO9l4uiP7L2OmWkCPtMHXBqS7TRzM4s=";
            subPackages = [ "./cmd/wasmd" ];
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "wasmd";
          }
          // (
            let
              # libwasmvm = self'.packages.libwasmvm-2_1_2;
              libwasmvm_aarch64 = pkgs.stdenv.mkDerivation {
                name = "libwasmvm";
                src = pkgs.fetchurl {
                  url = "https://github.com/CosmWasm/wasmvm/releases/download/v2.1.2/libwasmvm_muslc.aarch64.a";
                  hash = "sha256-CIHFtGPoniKbBjcOnilhrsClxjZ3LVFCxo01FWRGSmY=";
                };
                dontUnpack = true;
                buildPhase = ''
                  mkdir -p $out/lib/
                  cp $src $out/lib/libwasmvm_muslc.aarch64.a
                '';
              };
              libwasmvm_x86_64 = pkgs.stdenv.mkDerivation {
                name = "libwasmvm";
                src = pkgs.fetchurl {
                  url = "https://github.com/CosmWasm/wasmvm/releases/download/v2.1.2/libwasmvm_muslc.x86_64.a";
                  hash = "sha256-WOH2v6ie45DLmrxppbwSYCmkl/4J3TmfOKgtDYb+le8=";
                };
                dontUnpack = true;
                buildPhase = ''
                  mkdir -p $out/lib/
                  cp $src $out/lib/libwasmvm_muslc.x86_64.a
                '';
              };
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  libwasmvm_aarch64
                  libwasmvm_x86_64
                ];
                tags = [
                  "muslc"
                  "netgo"
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -static -L${libwasmvm_aarch64}/lib -L${libwasmvm_x86_64}/lib'"
                  "-X github.com/cosmos/cosmos-sdk/version.Name=wasm"
                  "-X github.com/cosmos/cosmos-sdk/version.AppName=wasmd"
            		  "-X github.com/CosmWasm/wasmd/app.Bech32Prefix=wasm"
            		  "-X github.com/cosmos/cosmos-sdk/version.Version=0.53.0"
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
