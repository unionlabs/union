{ self, ... }: {
  perSystem = { pkgs, crane, ... }:
    {
      packages = {
        tera = crane.lib.buildPackage {
          name = "tera";  
          src = pkgs.fetchFromGitHub {
            name = "tera";
            owner = "chevdor";
            repo = "tera-cli";
            rev = "b805115917127ca5467978b872d031ce1fb734e7";
            sha256 = null;
          };
          meta.mainProgram = "tera";
        };
      };
    };
}
