{
  pkgs,
  lib,
  rust,
  pkgsUnstable,
  movefmt,
}:
{
  package = pkgs.treefmt;
  projectRootFile = "treefmt.nix";
  programs = {
    gofmt = {
      enable = true;
      package = pkgs.go_1_23;
    };
    rustfmt = {
      enable = true;
      package = rust.toolchains.dev;
    };
    taplo.enable = true;
    biome = {
      enable = true;
      package = pkgs.biome;
    };
    yamlfmt = {
      enable = true;
      package = pkgs.yamlfmt;
    };
    mdformat = {
      enable = true;
      package = pkgs.mdformat;
    };
    shellcheck = {
      enable = true;
      package = pkgs.shellcheck;
    };
    nixfmt-rfc-style = {
      enable = true;
      package = pkgs.nixfmt-rfc-style;
    };
    statix = {
      enable = true;
      package = pkgs.statix;
    };
    deadnix = {
      enable = true;
      package = pkgs.deadnix;
    };
  };
  settings = {
    formatter = {
      nixfmt-rfc-style = {
        options = [ ];
        includes = [ "*.nix" ];
      };
      statix.options = [ "explain" ];
      mdformat.options = [ "--number" ];
      deadnix.options = [ "--no-lambda-pattern-names" ];
      shellcheck.options = [
        "--shell=bash"
        "--check-sourced"
      ];
      yamlfmt.options = [
        "-formatter"
        "retain_line_breaks=true"
      ];
      biome =
        let
          biomeJsonConfig = builtins.fromJSON (builtins.readFile ./biome.json);
        in
        {
          options = [
            "check"
            "--config-path"
            "./biome.json"
          ];
          includes = biomeJsonConfig.files.include;
        };
      sort =
        let
          filesToSort = [ "dictionary.txt" ];
        in
        {
          command =
            let
              sort = pkgs.symlinkJoin {
                name = "sort";
                paths = [ pkgs.coreutils ];
                buildInputs = [ pkgs.makeWrapper ];
                postBuild = ''
                  wrapProgram $out/bin/sort \
                    --set LC_ALL C \
                    --set LC_COLLATE C
                '';
              };
            in
            "${sort}/bin/sort";
          options = [ "-uo" ] ++ filesToSort;
          includes = filesToSort;
        };
      forge = {
        command =
          let
            forge = pkgs.symlinkJoin {
              name = "forge";
              paths = [ pkgs.foundry-bin ];
              buildInputs = [ pkgs.makeWrapper ];
              postBuild = ''
                wrapProgram $out/bin/forge \
                  --set FOUNDRY_CONFIG "${./foundry.toml}"
              '';
            };
          in
          "${forge}/bin/forge";
        options = [ "fmt" ];
        includes = [ "*.sol" ];
      };
      movefmt = {
        command = "${movefmt}/bin/movefmt";
        options = [ ];
        includes = [ "*.move" ];
      };
      deployments-json =
        let
          filesToFormat = [
            "deployments/channels.json"
            "deployments/editions.json"
          ];
        in
        {
          command = lib.getExe (
            pkgs.writeShellScriptBin "format-deployments" ''
              # sort with jq
              ${lib.getExe pkgs.jq} . "$1" -S | ${lib.getExe' pkgs.moreutils "sponge"} "$1"

              # format using biome
              ${lib.getExe pkgs.biome} format --config-path ./biome.json --write "$1"
            ''
          );
          options = [ ];
          includes = filesToFormat;
        };
    };
    global = {
      hidden = true;
      excludes = [
        "_/**"
        "*.ttf"
        "*.png"
        "*.prv"
        "*.bin"
        "*.jpg"
        "*.svg"
        "*.jpeg"
        "*.lock"
        ".git/**"
        "*.woff2"
        "*.lockb"
        ".ignore"
        "LICENSE"
        "LICENSE*"
        "**/*.ico"
        "**/*.zip"
        "**/.npmrc"
        "**/LICENSE"
        ".gitignore"
        "CODEOWNERS"
        "*.template"
        ".gitignore"
        "**/.sqlx/**"
        "**/vendor/**"
        "*.splinecode"
        "**/.gitignore"
        ".gitattributes"
        "**/testdata/**"
        "**/testswap/**"
        "**/generated/**"
        ".github/**/*.sh"
        ".github/**/*.md"
        "**/.gitattributes"
        "uniond/docs/static/**"
        ".git-blame-ignore-revs"
        "cosmwasm/cw20-base/**"
      ];
    };
  };
}
