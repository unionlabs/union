_: {
  perSystem =
    { pkgs, crane, ... }:
    let
      name = "tera";
    in
    {
      _module.args.tera = crane.lib.buildPackage {
        inherit name;
        version = "0.2.4";
        src = pkgs.fetchFromGitHub {
          inherit name;
          owner = "chevdor";
          repo = "tera-cli";
          rev = "fbccb741db347aa6c85a7d14d98bdc83ddc2dedd";
          sha256 = "sha256-ZBlxikPa92qqKTCyOzFT6pVmNnGxw+0G0XTwzH/ST3w=";
        };
        meta.mainProgram = "tera";
      };
    };
}
