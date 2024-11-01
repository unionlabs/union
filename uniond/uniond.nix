{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      nix-filter,
      gitRev,
      uniondBundleVersions,
      goPkgs,
      mkCi,
      ...
    }:
    let
      libwasmvm = self'.packages.libwasmvm-2_3_1;
      CGO_CFLAGS = "-I${self'.packages.libblst}/include -I${self'.packages.libblst.src}/src -I${self'.packages.libblst.src}/build -I${self'.packages.bls-eth.src}/bls/include -O";
      CGO_LDFLAGS = "-z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib -L${self'.packages.bls-eth}/lib -s -w";

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
        bls-eth =
          let
            isAarch64 = (builtins.head (pkgs.lib.splitString "-" system)) == "aarch64";
          in
          pkgs.pkgsStatic.stdenv.mkDerivation {
            pname = "bls-eth";
            version = inputs.bls-eth-go.shortRev;
            src = inputs.bls-eth-go;
            nativeBuildInputs = [
              pkgs.pkgsStatic.nasm
            ] ++ (pkgs.lib.optionals isAarch64 [ pkgs.llvmPackages_9.libcxxClang ]);
            installPhase = ''
              mkdir -p $out/lib
              ls -al bls/lib/linux/
              mv bls/lib/linux/${if isAarch64 then "arm64" else "amd64"}/*.a $out/lib
            '';
            enableParallelBuilding = true;
            doCheck = true;
          };

        # Statically link on Linux using `pkgsStatic`, dynamically link on Darwin using normal `pkgs`.
        uniond =
          (if pkgs.stdenv.isLinux then goPkgs.pkgsStatic.buildGo123Module else goPkgs.buildGo123Module)
            (
              {
                name = "uniond";
                src = nix-filter {
                  name = "uniond-source";
                  root = ./.;
                  exclude = [
                    (nix-filter.matchExt "nix")
                    (nix-filter.matchExt "md")
                  ];
                };
                vendorHash = "sha256-q4LfdIV0H/Gv/elNpTGVPkY9x0NP4tg6Wn2V7dZuwHA=";
                doCheck = true;
                meta.mainProgram = "uniond";
              }
              // (
                if pkgs.stdenv.isLinux then
                  {
                    inherit CGO_CFLAGS;
                    inherit CGO_LDFLAGS;
                    # Statically link if we're on linux
                    nativeBuildInputs = [ pkgs.musl ];
                    ldflags = [
                      "-checklinkname=0"
                      "-linkmode external"
                      "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
                      "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
                    ];
                  }
                else if pkgs.stdenv.isDarwin then
                  {
                    # Dynamically link if we're on darwin by wrapping the program
                    # such that the DYLD_LIBRARY_PATH includes libwasmvm
                    buildInputs = [ pkgs.makeWrapper ];
                    postFixup = ''
                      wrapProgram $out/bin/uniond \
                      --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
                    '';
                    ldflags = [
                      "-checklinkname=0"
                      "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
                      "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
                    ];
                  }
                else
                  { }
              )
            );

        uniond-release = mkCi false (
          self'.packages.uniond.overrideAttrs (old: {
            ldflags = old.ldflags ++ [
              "-X github.com/cosmos/cosmos-sdk/version.Name=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.AppName=uniond"
              "-X github.com/cosmos/cosmos-sdk/version.BuildTags=${system}"
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
