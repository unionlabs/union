test:
	go test
	dot -Tpng < TestCompositeWithUnusedIOSameGraph.dot > TestCompositeWithUnusedIOSameGraph.png && open TestCompositeWithUnusedIOSameGraph.png
	dot -Tpng < TestExampleSubsystemSameGraph.dot > TestExampleSubsystemSameGraph.png && open TestExampleSubsystemSameGraph.png
	dot -Tsvg < TestExampleSubsystemExternalGraph.dot > TestExampleSubsystemExternalGraph.svg && open TestExampleSubsystemExternalGraph.svg
	dot -Tsvg < subsystem.dot > subsystem.svg && open subsystem.svg
	dot -Tsvg < subsystem2.dot > subsystem2.svg && open subsystem2.svg