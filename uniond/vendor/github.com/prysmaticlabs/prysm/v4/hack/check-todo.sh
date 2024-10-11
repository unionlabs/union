#!/bin/bash

# Continuous integration script to check that TODOs are in the correct format
OUTPUT="$(grep -PrinH '(?<!context\.)todo(?!\(#{0,1}\d+\))' --include ./**/*.go --exclude ./*site_data.go --exclude ./*mainnet_config.go)";
if [ "$OUTPUT" != "" ] ;
then
    echo "Invalid TODOs found. Failing." >&2;
    echo "$OUTPUT" >&2;
    exit 1;
fi

while read -r line ; do
linenum=$(expr "$line" : '^\([0-9]*:\)')
issueNum=${line//$linenum}
issueState=$(curl https://api.github.com/repos/prysmaticlabs/prysm/issues/"$issueNum" | grep -o '"state":"closed"');

if [ "$issueState" != "" ];
then
    echo "Issue referenced has already been closed" >&2;
    echo "Issue Number: $issueNum" >&2;
    exit 1;
fi
done < <(grep -PrinH -o -h '(?<!context\.)todo\(#{0,1}\K(\d+)' --include ./*.go)
