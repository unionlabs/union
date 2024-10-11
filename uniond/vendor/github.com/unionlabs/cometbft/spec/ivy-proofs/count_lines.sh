#!/bin/bash

r='^\s*$\|^\s*\#\|^\s*\}\s*$\|^\s*{\s*$' # removes comments and blank lines and lines that contain only { or }
N1=`cat tendermint.ivy domain_model.ivy network_shim.ivy | grep -v $r'\|.*invariant.*' | wc -l`
N2=`cat abstract_tendermint.ivy | grep "observed_" | wc -l` # the observed_* variables specify the observations of the nodes
SPEC_LINES=`expr $N1 + $N2`
echo "spec lines: $SPEC_LINES"
N3=`cat abstract_tendermint.ivy | grep -v $r'\|.*observed_.*' | wc -l`
N4=`cat accountable_safety_1.ivy | grep -v $r | wc -l`
PROOF_LINES=`expr $N3 + $N4`
echo "proof lines: $PROOF_LINES"
RATIO=`bc <<< "scale=2;$PROOF_LINES / $SPEC_LINES"`
echo "proof-to-code ratio for the accountable-safety property: $RATIO"
