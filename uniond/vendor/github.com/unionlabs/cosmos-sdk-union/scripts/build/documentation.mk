#? godocs: Generate go doc
godocs:
	@echo "--> Wait a few seconds and visit http://localhost:6060/pkg/github.com/cosmos/cosmos-sdk/types"
	go install golang.org/x/tools/cmd/godoc@latest
	godoc -http=:6060
