{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', crane, system, ensureAtRepositoryRoot, dbg, ... }: {
    packages = {
      starsd = goPkgs.pkgsStatic.buildGoModule ({
        name = "starsd";
        src = inputs.stargaze;
        vendorHash = "sha256-yg2RT1NnUaqlU8Gvlx5ZxHotSMZbBnicsAj5Jzi0BXo=";
        doCheck = false;
        doInstallCheck = false;
        meta.mainProgram = "starsd";
        # CGO_ENABLED = 0;
        subPackages = [ "./cmd/starsd" ];
        buildTags = [ "netgo" ];
      } // (
        let libwasmvm = self'.packages.libwasmvm;
        in if pkgs.stdenv.isLinux then {
          # Statically link if we're on linux
          nativeBuildInputs = [ pkgs.musl libwasmvm ];
          ldflags = [
            "-linkmode external"
            "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib'"
          ];
        }
        # else if pkgs.stdenv.isDarwin then {
        #   # Dynamically link if we're on darwin by wrapping the program
        #   # such that the DYLD_LIBRARY_PATH includes libwasmvm
        #   buildInputs = [ pkgs.makeWrapper libwasmvm ];
        #   postFixup = ''
        #     wrapProgram $out/bin/wasmd \
        #     --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
        #   '';
        # } else
        else
          { }
      ));

      # cspell:disable-next-line
      # this is broken right now due to an outdated transitive dependency to `bnum` (https://github.com/public-awesome/launchpad/blob/b584245bcc7ff07d9b384df1ccc4e3e2466d92db/Cargo.lock#L507-L511)
      # if this is still not fixed by the time we need this contract, we can build this with a different nightly version (channel).
      # sg721 = crane.buildRemoteWasmContract {
      #   src = inputs.public-awesome-launchpad;
      #   version = inputs.public-awesome-launchpad.rev;
      #   package = "sg721";
      # };
    };
  };
}

