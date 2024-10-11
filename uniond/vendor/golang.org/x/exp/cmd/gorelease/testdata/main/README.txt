Module example.com/main is used to test changes in main packages.
Main packages aren't importable, so changes to exported functions should not
be reported. But we should still report when packages are added or deleted.
