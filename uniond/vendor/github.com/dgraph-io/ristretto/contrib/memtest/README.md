memtest tests the effect of the C memory allocator. The default version uses Calloc from the stdlib.

If the program is built using the `jemalloc` build tag, then the allocator used will be jemalloc.

# Monitoring #

To monitor the memory use of this program, the following bash snippet is useful:

```
while true; do
ps -C memtest -o vsz=,rss= >> memphys.csv
sleep 1
done
```

This is of course contingent upon the fact that the binary of this program is called `memtest`.
