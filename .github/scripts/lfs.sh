OUTPUT=$(git ls-files | git check-attr --stdin filter | \
          awk -F': ' '$3 ~ /lfs/ { print $1}' | \
          xargs -L1 sh -c 'git cat-file blob "HEAD:$0" | \
          git lfs pointer --check --stdin || { echo "$0"; false; }')

if [ -z "$OUTPUT" ];
then
    exit 0
else
    echo "Found files that should be in LFS"
    echo "$OUTPUT"
    exit 1
fi  