package mux_test

import (
	"fmt"
	"github.com/gorilla/mux"
)

// This example demonstrates building a dynamic URL using
// required vars and values retrieve from another source
func ExampleRoute_GetVarNames() {
	r := mux.NewRouter()

	route := r.Host("{domain}").
		Path("/{group}/{item_id}").
		Queries("some_data1", "{some_data1}").
		Queries("some_data2_and_3", "{some_data2}.{some_data3}")

	dataSource := func(key string) string {
		return "my_value_for_" + key
	}

	varNames, _ := route.GetVarNames()

	pairs := make([]string, 0, len(varNames)*2)

	for _, varName := range varNames {
		pairs = append(pairs, varName, dataSource(varName))
	}

	url, err := route.URL(pairs...)
	if err != nil {
		panic(err)
	}
	fmt.Println(url.String())
}
