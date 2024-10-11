#!/bin/bash

# returns non-zero error code if any proof fails

success=0
log_dir=$(cat /dev/urandom | tr -cd 'a-f0-9' | head -c 6)
cmd="ivy_check seed=$RANDOM"
mkdir -p output/$log_dir

echo "Checking classic safety:"
res=$($cmd classic_safety.ivy | tee "output/$log_dir/classic_safety.txt" | tail -n 1)
if [ "$res" = "OK" ]; then
  echo "OK"
else
  echo "FAILED"
  success=1
fi

echo "Checking accountable safety 1:"
res=$($cmd accountable_safety_1.ivy | tee "output/$log_dir/accountable_safety_1.txt" | tail -n 1)
if [ "$res" = "OK" ]; then
  echo "OK"
else
  echo "FAILED"
  success=1
fi

echo "Checking accountable safety 2:"
res=$($cmd complete=fo accountable_safety_2.ivy | tee "output/$log_dir/accountable_safety_2.txt" | tail -n 1)
if [ "$res" = "OK" ]; then
  echo "OK"
else
  echo "FAILED"
  success=1
fi

echo
echo "See ivy_check output in the output/ folder"
exit $success
