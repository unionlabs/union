# The protobuf generation process is based on:
#
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/contrib/devtools/Dockerfile
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/Makefile#L401
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/scripts/protocgen.sh
#
{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', proto, ibc-go, ensureAtRepositoryRoot, mkCi, dbg, ... }: {
    packages =
      let
        CGO_CFLAGS = "";
        CGO_LDFLAGS = "-z noexecstack -static -L${pkgs.musl}/lib -L${self'.packages.libwasmvm}/lib";

        polard = (if pkgs.stdenv.isLinux then
          goPkgs.pkgsStatic.buildGo121Module
        else
          goPkgs.buildGo121Module) ({
          name = "polard";
          src = "${inputs.berachain}/e2e/testapp";
          vendorHash = "sha256-izghkLhUAdNnh+bvF6qvj6h9F4IIt8Twckp5PnJAMxs=";
          # times out for some reason
          doCheck = false;
          tags = [
            "netgo"
            "pebbledb"
          ];
          meta.mainProgram = "polard";
        } // (
          if pkgs.stdenv.isLinux then {
            inherit CGO_CFLAGS;
            inherit CGO_LDFLAGS;
            # Statically link if we're on linux
            nativeBuildInputs = [ pkgs.musl ];
            ldflags = [
              "-linkmode external"
              "-X github.com/cosmos/cosmos-sdk/version.Name=polard"
              "-X github.com/cosmos/cosmos-sdk/version.AppName=polard"
              ''-X "github.com/cosmos/cosmos-sdk/version.BuildTags=netgo pebbledb,"''
              "-s"
              "-w"
            ];
          } else if pkgs.stdenv.isDarwin then {
            # Dynamically link if we're on darwin by wrapping the program
            # such that the DYLD_LIBRARY_PATH includes libwasmvm
            buildInputs = [ pkgs.makeWrapper ];
            postFixup = ''
              wrapProgram $out/bin/polard \
              --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ self'.packages.libwasmvm ])};
            '';
            ldflags = [
              "-X github.com/cosmos/cosmos-sdk/version.Name=polard"
              "-X github.com/cosmos/cosmos-sdk/version.AppName=polard"
              ''-X "github.com/cosmos/cosmos-sdk/version.BuildTags=netgo pebbledb,"''
              "-s"
              "-w"
            ];
          } else
            { }
        ));
      in
      {
        inherit polard;
      };
    checks = { };
  };
}

