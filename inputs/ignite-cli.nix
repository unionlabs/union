{ inputs, ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {
      ignite-cli = pkgs.buildGoModule rec {
        allowGoReference = true;
        patches = [ ./ignite-cli.patch ];
        nativeBuildInputs = [ pkgs.protobuf ];
        buildInputs = [ pkgs.protobuf self'.packages.swagger-combine ];
        name = "ignite-cli";
        src = inputs.ignite-cli-src;
        vendorSha256 = "sha256-TWOxdq2LTnxd718Ra0viD1z2tBnNmcN92A1wpX97xtc=";
        doCheck = false;
        ldflags = ''
          -X github.com/ignite/cli/ignite/version.Head=${src.rev}
          -X github.com/ignite/cli/ignite/version.Version=v0.26.1
          -X github.com/ignite/cli/ignite/version.Date=${builtins.toString (src.lastModified)}
        '';
      };
    };
  };
}
