_: {
  perSystem =
    {
      pkgs,
      self',
      system,
      nix-filter,
      gitRev,
      uniondBundleVersions,
      goPkgs,
      mkCi,
      buildGoApplication,
      ...
    }:
    let
      libwasmvm = self'.packages.libwasmvm-2_2_1;
      CGO_LDFLAGS = "-lrocksdb -pthread -lstdc++ -ldl -z noexecstack -static -L${goPkgs.musl}/lib -L${libwasmvm}/lib -s -w";

      mkUniondImage =
        uniond:
        pkgs.dockerTools.buildImage {
          name = "uniond";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils
              pkgs.cacert
              uniond
            ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ "uniond" ];
            Cmd = [ "start" ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
    in
    {
      packages = {
        # Statically link on Linux using `pkgsStatic`, dynamically link on Darwin using normal `pkgs`.
        uniond =
          buildGoApplication {
            inherit CGO_LDFLAGS;
            CGO_ENABLED = "1";
            name = "uniond";
            pwd = ./.;
            src = nix-filter {
              name = "uniond-source";
              root = ./.;
              exclude = [
                (nix-filter.matchExt "nix")
                (nix-filter.matchExt "md")
              ];
            };
            modules = ./gomod2nix.toml;
            # nativeBuildInputs = [
            #   goPkgs.musl
            #   libwasmvm
            # ];
            buildInputs = [
              goPkgs.musl
              libwasmvm
              pkgs.rocksdb
            ];
            doCheck = false;
            tags = [ "ledger" "muslc" "netgo" ];
            ldflags = [
              "-linkmode external"
              "-extldflags \"-Wl,-z,muldefs -static\""
              "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.BuildTags=ledger,muslc,netgo"
            ];
          };
          # (if pkgs.stdenv.isLinux then goPkgs.pkgsStatic.buildGo123Module else goPkgs.buildGo123Module)
          #   (
          #     {
          #       name = "uniond";
          #       src = nix-filter {
          #         name = "uniond-source";
          #         root = ./.;
          #         exclude = [
          #           (nix-filter.matchExt "nix")
          #           (nix-filter.matchExt "md")
          #         ];
          #       };
          #       vendorHash = "sha256-V69IzwiwgRb6tiJg5S2ca7DBYGNtndc39DEtpxJtN/g=";
          #       doCheck = true;
          #       meta.mainProgram = "uniond";
          #     }
          #     // (
          #       if pkgs.stdenv.isLinux then
          #         {
          #           inherit CGO_LDFLAGS;
          #           nativeBuildInputs = [
          #             goPkgs.musl
          #             libwasmvm
          #             pkgs.stdenv.cc.cc.lib
          #             pkgs.gccgo
          #             pkgs.gcc
          #           ];
          #           tags = [ "ledger" "netgo" ];
          #           CGO_ENABLE = "1";
          #           ldflags = [
          #             "-linkmode external"
          #             "-extldflags \"-Wl,-z,muldefs -static\""
          #             "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
          #             "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
          #             "-X github.com/cosmos/cosmos-sdk/version.BuildTags=ledger,netgo"
          #           ];
          #         }
          #       else if pkgs.stdenv.isDarwin then
          #         {
          #           # Dynamically link if we're on darwin by wrapping the program
          #           # such that the DYLD_LIBRARY_PATH includes libwasmvm
          #           buildInputs = [ pkgs.makeWrapper ];
          #           postFixup = ''
          #             wrapProgram $out/ bin/uniond \
          #             --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
          #           '';
          #           CGO_ENABLE = "1";
          #           ldflags = [
          #             "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
          #             "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
          #             "-X github.com/cosmos/cosmos-sdk/version.BuildTags=ledger"
          #           ];
          #         }
          #       else
          #         { }
          #     )
          #   );

        uniond-release = mkCi false (
          self'.packages.uniond.overrideAttrs (old: {
            ldflags = old.ldflags ++ [
              "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.BuildTags=${system},ledger"
              "-X github.com/cosmos/cosmos-sdk/version.Commit=${gitRev}"
              "-X github.com/cosmos/cosmos-sdk/version.Version=${uniondBundleVersions.last}"
            ];
          })
        );

        uniond-image = mkUniondImage self'.packages.uniond;

        uniond-release-image = mkUniondImage self'.packages.uniond-release;
      };
    };
}
