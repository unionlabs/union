# Release Notes v0.5.8

This release fixes the security issue #35. The readUvarint function
would run infinitely given specific input. The function is now
terminating if more than 10 bytes of input have been read. The behavior
is tested.

Many thanks to Github user 0xdecaf for reporting the issue.
