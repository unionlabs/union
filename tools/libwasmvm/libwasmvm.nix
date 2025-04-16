_:
let
  # TODO: populate when needed
  hashes = {
    "1.5.2" = {
      "x86_64-linux" = "sha256-5mCjjvspMLNO5vawuxJzCtzLBAtqtwG4+C80RTpCauc=";
      "aarch64-linux" = "sha256-54siTBWWSBejt1pA5ZiCtNDgb9BVs5UU1hZGaJzvjG4=";
      "aarch64-darwin" = "";
    };
    "1.5.8" = {
      "x86_64-linux" = "sha256-KxmfNW9j6LHKmKNbpZbByYBwgeLFymwHIxs0GH2OCO8=";
      "aarch64-linux" = "sha256-DjQizkaip8JRrd7qA1TOrDaCx+FwFA8sDsSSiPD9IYg=";
      "aarch64-darwin" = "";
    };
    "2.1.2" = {
      "x86_64-linux" = "sha256-WOH2v6ie45DLmrxppbwSYCmkl/4J3TmfOKgtDYb+le8=";
      "aarch64-linux" = "sha256-CIHFtGPoniKbBjcOnilhrsClxjZ3LVFCxo01FWRGSmY=";
      "aarch64-darwin" = "";
    };
    "2.1.3" = {
      "x86_64-linux" = "sha256-jasIQ0pf5Xpvu8uAQXlLw8MYRtMfj/X7NT7nTg/NMJM=";
      "aarch64-linux" = "sha256-+upOFTkOBG0sqEQcIaiNulb5oDY/ksXZQBXfCsbaHy0=";
      "aarch64-darwin" = "";
    };
    "2.1.4" = {
      "x86_64-linux" = "sha256-pKPQmzb6u2WxGdW6I0QsI2lEAfy+5EUf5rfiLjJaS6w=";
      "aarch64-linux" = "sha256-CQuXZBFX+uGuReftNooajAkfP+9nlY07x8L6fnxUtrQ=";
      "aarch64-darwin" = "";
    };
    "2.2.1" = {
      "x86_64-linux" = "sha256-s711XvrA/znAG1m4EQ+WHEiqPrk1iAcdemKCcMwfIyY=";
      "aarch64-linux" = "sha256-umy122sUomXIVWMmwEWICQjbmx0v+11KqfCawJskzsw=";
      "aarch64-darwin" = "";
    };
  };
in
{
  perSystem =
    {
      pkgs,
      self',
      crane,
      rust,
      system,
      ensureAtRepositoryRoot,
      srcWithVendoredSources,
      dbg,
      ...
    }:
    let
      fetchReleaseArtifact =
        {
          version,
          noPrefix ? false,
        }:
        let
          prefix = builtins.head (pkgs.lib.strings.splitString "-" system);
          artifact = pkgs.fetchurl {
            # TODO: incompatible with darwin we need to cut the `muslc` prefix
            url = "https://github.com/CosmWasm/wasmvm/releases/download/v${version}/libwasmvm_muslc.${prefix}.a";
            hash = hashes.${version}.${system};
          };
        in
        pkgs.runCommand "libwasmvm-${version}-${system}" { } (
          ''
            mkdir -p $out/lib
          ''
          + (
            if noPrefix then
              ''
                cp ${artifact} $out/lib/libwasmvm_muslc.a
              ''
            else
              ''
                cp ${artifact} $out/lib/libwasmvm_muslc.${prefix}.a
              ''
          )
        );
    in
    {
      packages.libwasmvm-1_5_2 = fetchReleaseArtifact {
        version = "1.5.2";
      };
      packages.libwasmvm-1_5_8 = fetchReleaseArtifact {
        version = "1.5.8";
        noPrefix = true;
      };
      packages.libwasmvm-2_1_2 = fetchReleaseArtifact {
        version = "2.1.2";
      };
      packages.libwasmvm-2_1_3 = fetchReleaseArtifact {
        version = "2.1.3";
      };
      packages.libwasmvm-2_1_4 = fetchReleaseArtifact {
        version = "2.1.4";
      };
      packages.libwasmvm-2_2_1 = fetchReleaseArtifact {
        version = "2.2.1";
      };
    };
}
