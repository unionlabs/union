{ self, ... }: {
  perSystem = { pkgs, crane, ... }:
    let 
        name = "tera";
    in
    {
      packages = {
        tera = crane.lib.buildPackage {
          name = name;
          version = "0.2.4";  
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
