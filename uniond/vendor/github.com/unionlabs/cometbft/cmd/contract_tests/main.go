package main

import (
	"fmt"
	"strings"

	"github.com/snikch/goodman/hooks"
	"github.com/snikch/goodman/transaction"
)

func main() {
	// This must be compiled beforehand and given to dredd as parameter, in the meantime the server should be running
	h := hooks.NewHooks()
	server := hooks.NewServer(hooks.NewHooksRunner(h))
	h.BeforeAll(func(t []*transaction.Transaction) {
		fmt.Println(t[0].Name)
	})
	h.BeforeEach(func(t *transaction.Transaction) {
		if strings.HasPrefix(t.Name, "Tx") ||
			// We need a proper example of evidence to broadcast
			strings.HasPrefix(t.Name, "Info > /broadcast_evidence") ||
			// We need a proper example of path and data
			strings.HasPrefix(t.Name, "ABCI > /abci_query") ||
			// We need to find a way to make a transaction before starting the tests,
			// that hash should replace the dummy one in the openapi file
			strings.HasPrefix(t.Name, "Info > /tx") {
			t.Skip = true
			fmt.Printf("%s Has been skipped\n", t.Name)
		}
	})
	server.Serve()
	defer server.Listener.Close()
	fmt.Print("FINE")
}
