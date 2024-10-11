## go-digest release process

1. Create release pull request with release notes and updated versions.

   1. Compile release notes detailing features added since the last release and
      add release template file to `releases/` directory. The template is defined
      by containerd's release tool but refer to previous release files for style
      and format help. Name the file using the version, for rc add an `-rc` suffix.
      When moving from rc to final, the rc file may just be renamed and updated.
      See [release-tool](https://github.com/containerd/release-tool)

   2. Update the `.mailmap` files for commit authors which have multiple email addresses in the commit log.
      If it is not clear which email or name the contributor might want used in the release notes, reach
      out to the contributor for feedback. NOTE: real names should be used whenever possible. The file is
      maintained by manually adding entries to the file.
      - e.g. `Real Name <preferred@email.com> Other Name <other@email.com>`

   3. Before opening the pull request, run the release tool using the new release notes.
      Ensure the output matches what is expected, including contributors, change log,
      dependencies, and visual elements such as spacing. If a contributor is duplicated,
      use the emails outputted by the release tool to update the mailmap then re-run. The
      goal of the release tool is that is generates release notes that need no
      alterations after it is generated.

2. Create tag

   1. Choose tag for the next release, go-digest uses semantic versioning and
      expects tags to be formatted as `vx.y.z[-rc.n]`.

   2. Generate release notes (using a temp file may be helpful).
      - e.g. `release-tool -l -d -n -t v1.0.0 ./releases/v1.0.0.toml > /tmp/v1.0.0-notes`

   3. Create tag using the generated release notes.
      - e.g. `git tag --cleanup=whitespace -s v1.0.0 -F /tmp/v1.0.0-notes`

   4. Verify tag (e.g. `git show v1.0.0`), it may help to compare the new tag against previous.

3. Push tag and Github release

   1. Push the tag to `git@github.com:opencontainers/go-digest.git`.
      NOTE: this will kick off CI building of the release binaries.

   2. Create the Github release using the `Tag version` which was just pushed. Use the first
      line outputted from the release tool as the `Release title` and the remainder of the
      output for the description. No alteration of the release output should be needed.
      Ensure `pre-release` is checked if an `-rc`.
      NOTE: This should be done immediately after pushing the tag, otherwise CI may create the release
      when the binaries are pushed.

4. Promote on Slack, Twitter, mailing lists, etc
