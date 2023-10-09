{ ... }: {
  perSystem = { pkgs, crane, ... }:
    let
      name = "sqlx-cli";
    in
    {
      packages = {
        sqlx-cli = crane.lib.buildPackage {
          name = name;
          version = "0.7.1";
          cargoExtraArgs = "-p sqlx-cli";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          src = pkgs.fetchFromGitHub {
            inherit name;
            owner = "launchbadge";
            repo = "sqlx";
            rev = "b1387057e5e6c6b72eacd01f491cb45854616502";
            sha256 = "sha256-jCMDJuE7iYCAWgIDRq4KVGrwbV3TM0Ws9GiFxFn+hVU=";
          };
          meta.mainProgram = "sqlx";
        };
      };
    };
}
