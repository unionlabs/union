#! /bin/bash

# Usage:
#   `./authors.sh`
#		Print a list of all authors who have committed to develop since master.
#
#   `./authors.sh <email address>`
#		Lookup the email address on Github and print the associated username

author=$1

if [[ "$author" == "" ]]; then
	git log master..develop | grep Author | sort | uniq
else
	curl -s "https://api.github.com/search/users?q=$author+in%3Aemail&type=Users&utf8=%E2%9C%93" | jq .items[0].login
fi
