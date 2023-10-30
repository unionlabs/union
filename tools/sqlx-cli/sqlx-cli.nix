{ ... }: {
  perSystem = { pkgs, rust, srcWithVendoredSources, ... }:
    let
      name = "sqlx-cli";
      sqlx = pkgs.fetchFromGitHub {
        name = "sqlx";
        owner = "launchbadge";
        repo = "sqlx";
        rev = "v0.7.1";
        sha256 = "sha256-567/uJPQhrNqDqBF/PqklXm2avSjvtQsddjChwUKUCI=";
      };
    in
    {
      _module.args.sqlxCliCargoToml = "${sqlx}/${name}/Cargo.toml";

      packages.sqlx-cli =
        pkgs.stdenv.mkDerivation {
          pname = name;
          version = "0.7.1";
          nativeBuildInputs = [ pkgs.pkg-config ];

          buildPhase = "cargo build --release --locked --offline -p ${name}";
          installPhase = ''
            mkdir -p $out/bin
            mv target/release/sqlx $out/bin/sqlx
          '';

          buildInputs = [ rust.toolchains.nightly pkgs.openssl ];

          src = srcWithVendoredSources { inherit name; originalSrc = "${sqlx}"; };

          meta.mainProgram = "sqlx";
        };
    };
}
