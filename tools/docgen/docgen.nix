{ self, ... }: {
  perSystem = { pkgs, self', lib, ... }:
    let
      tera = lib.meta.getExe self'.packages.tera;
      rootFile = "flake.nix";
      doc_comment = "<!-- GENERATED: DO NOT EDIT -->";
    in
    {
      packages = {
        docgen = pkgs.writeShellApplication {
          name = "docgen";
          runtimeInputs = [ tera ];
          text = ''
            if ! test -f "${rootFile}"; then
                echo "Error: please run docgen from the root of the repository"
                exit 1
            fi

            echo '{"doc_comment": "${doc_comment}"}' | ${tera} --template unionvisor/docs/README.md --include-path unionvisor -s -o unionvisor/README.md > /dev/null 2>&1
          '';
        };
      };

      checks = {
        docgen-updated = pkgs.stdenv.mkDerivation {
          name = "docgen-check";
          src = ../../.;
          buildInputs = [ pkgs.git self'.packages.docgen ];
          doCheck = true;
          checkPhase = ''
            set -e
            PRJ=$TMP/project
            cp -r ${self} $PRJ
            chmod -R a+w $PRJ
            cd $PRJ
            export HOME=$TMPDIR
            cat > $HOME/.gitconfig <<EOF
            [user]
            name = Nix
            email = nix@localhost
            [init]
            defaultBranch = main
            EOF
            git init
            git add .
            git commit -m init --quiet
            export LANG=C.UTF-8
            export LC_ALL=C.UTF-8
            ${lib.meta.getExe self'.packages.docgen}
            git status
            git --no-pager diff --exit-code
            touch $out
          '';
        };
      };
    };
}
