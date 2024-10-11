The .go files in this directory are split into two packages, old and new.
They are syntactically valid Go so that gofmt can process them.

```
If a comment begins with:  Then:
old                        write subsequent lines to the "old" package
new                        write subsequent lines to the "new" package
both                       write subsequent lines to both packages
c                          expect a compatible error with the following text
i                          expect an incompatible error with the following text

```
