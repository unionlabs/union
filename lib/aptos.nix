_: {
  perSystem =
    {
      lib,
      pkgsUnstable,
      system,
      config,
      rust,
      crane,
      ...
    }:
    let
      throwBadSystem = throw "aptos cannot be built on system `${system}`";

      CARGO_BUILD_TARGET =
        if system == "aarch64-linux" then
          "aarch64-unknown-linux-musl"
        else if system == "x86_64-linux" then
          "x86_64-unknown-linux-musl"
        else if system == "aarch64-darwin" then
          "aarch64-apple-darwin"
        else if system == "x86_64-darwin" then
          "x86_64-apple-darwin"
        else
          throwBadSystem;

      rustToolchain = rust.mkNightly {
        channel = "1.78.0";
        targets = [ CARGO_BUILD_TARGET ];
      };

      craneLib = crane.lib.overrideToolchain rustToolchain;

      movefmt = craneLib.buildPackage rec {
        pname = "movefmt";
        version = "3201309e4cce72205994e32a4d45d1447db705e5";

        src = builtins.fetchGit {
          url = "https://github.com/movebit/movefmt";
          ref = "develop";
          rev = version;
        };

        doCheck = false;
      };

    in
    {
      packages = {
        inherit movefmt;
      };
    };
}
