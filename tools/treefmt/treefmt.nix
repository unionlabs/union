{ inputs, ... }: {
  perSystem = { pkgs, lib, config, ... }:
    let
      treefmt = inputs.treefmt-nix.lib.mkWrapper pkgs {
        options.programs.sort = {
          enable = lib.mkEnableOption "sort";
          package = lib.mkPackageOption pkgs "coreutils" { };
        };
        config = {
          programs = {
            nixpkgs-fmt.enable = true;
            gofmt.enable = true;
            rustfmt.enable = true;
            prettier.enable = true;
            sort.enable = true;
          };
          settings.formatter.sort = {
            command = "${pkgs.coreutils}/bin/sort";
            options = [ "-uo" "dictionary.txt" "dictionary.txt" ];
            includes = [ "dictionary.txt" ];
          };
          projectRootFile = "flake.nix";
          settings.global.excludes = [ "**/vendor/**" "**/foundry/lib/**" ];
        };
      };
    in
    {
      _module.args.treefmt = treefmt;
    };
}


