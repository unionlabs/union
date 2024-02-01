{ self, ... }: {
  perSystem = { pkgs, self', lib, config, tera, mkCi, system, ... }:
    let
      treefmt = config.treefmt.build.wrapper;
      teraBin = "${tera}/bin/tera";
      rootFile = "flake.nix";
      doc_comment = "<!-- GENERATED: DO NOT EDIT -->";
    in
    {
      packages = {
        docgen = mkCi false (pkgs.writeShellApplication {
          name = "docgen";
          runtimeInputs = [ teraBin treefmt ];
          text = ''
            if ! test -f "${rootFile}"; then
                echo "Error: please run docgen from the root of the repository"
                exit 1
            fi

            echo '{"doc_comment": "${doc_comment}"}' | ${teraBin} --template unionvisor/docs/README.md --include-path unionvisor -s -o unionvisor/README.md > /dev/null 2>&1
            treefmt > /dev/null 2>&1
          '';
        });
      };

      checks = {
        docgen-updated = mkCi (system == "x86_64-linux") (pkgs.stdenv.mkDerivation {
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
            ${self'.packages.docgen}/bin/docgen
            git status
            git --no-pager diff --exit-code
            touch $out
          '';
        });
      };
    };
}
