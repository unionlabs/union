{ ... }: {
  perSystem = { pkgs, rust, system, lib, dbg, inputs', goPkgs,...}: {
    packages = let
      interchaintest = goPkgs.buildGoModule {
        pname = "interchaintest";
        version = "v8.0.0";
        src = pkgs.fetchFromGitHub {
          owner = "strangelove-ventures";
          repo = "interchaintest";
          rev = "2f014d308bea4429169c94c4ba08759ce5e7be03";
          sha256 = "sha256-PY2S1ieVBmtb9OlF8YKke/Qlk/xZZdOe0TthmJlJyWg=";
        };
        vendorHash = "sha256-hJZ6klBzD6sbh6G7nwX+rEkh2e7Tq/3nLlOD4dlAvXk=";
        buildFlags = ["-c" "-o ./bin/interchaintest ./cmd/interchaintest"];
        ldflags = [ "-X github.com/strangelove-ventures/interchaintest/v8/internal/version.GitSha=$(shell git describe --always --dirty)" ];
        preBuild = ''
          export GOWORK=off
          rm -rf local-interchain
        '';
        doCheck = false;
      };
    in
    {
      interchaintest = interchaintest;
    };
  };
}
