# https://flake.parts/options/treefmt-nix#opt-perSystem.treefmt
{
  pkgs,
  rust,
  biome,
  goPkgs,
  unstablePkgs,
  movefmt,
}:

{
  package = unstablePkgs.treefmt;
  projectRootFile = "treefmt.nix";
  programs = {
    gofmt = {
      enable = true;
      package = unstablePkgs.go_1_23;
    };
    rustfmt = {
      enable = true;
      package = rust.toolchains.dev;
    };
    taplo.enable = true;
    biome = {
      enable = true;
      package = biome;
    };
    yamlfmt = {
      enable = true;
      package = unstablePkgs.yamlfmt;
    };
    mdformat = {
      enable = true;
      package = unstablePkgs.mdformat;
    };
    shellcheck = {
      enable = true;
      package = unstablePkgs.shellcheck;
    };
    nixfmt-rfc-style = {
      enable = true;
      package = unstablePkgs.nixfmt-rfc-style;
    };
    statix = {
      enable = true;
      package = unstablePkgs.statix;
    };
    deadnix = {
      enable = true;
      package = unstablePkgs.deadnix;
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
    };
    global = {
      hidden = true;
      excludes = [
        "_/**"
        "*.ttf"
        "*.png"
        ".git/**"
        "**/.sqlx/**"
        "**/vendor/**"
        "*.splinecode"
        "**/generated/**"
        ".github/**/*.sh"
        ".github/**/*.md"
        "uniond/docs/static/**"
      ];
    };
  };
}
