update-testdata-examples:
	go test . -test.update-golden
	dot -Tsvg testdata/example.dot > testdata/example.svg
	dot -Tsvg testdata/example_error.dot > testdata/example_error.svg
