{ inputs, ... }: {
  perSystem = { pkgs, ... }: {
    packages = {
      swagger-combine = pkgs.buildNpmPackage {
        pname = "swagger-combine";
        version = "10.0.9";
        src = inputs.swagger-combine-src;
        dontNpmBuild = true;
        npmDepsHash = "sha256-FZR8hefkqTwSZJMX4lzS4zk7iGXi0+zi0ol1ia3iLYs=";
      };
    };
  };
}
