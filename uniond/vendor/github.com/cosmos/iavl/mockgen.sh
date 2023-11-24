#!/usr/bin/env bash

mockgen_cmd="mockgen"
$mockgen_cmd -package mock -destination mock/db_mock.go github.com/cosmos/cosmos-db DB,Iterator,Batch