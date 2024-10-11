package sharedcache

func (c *Cache) WaitForWritesToComplete() {
	close(c.writeWorkers.tasksCh)
	c.writeWorkers.doneWaitGroup.Wait()
	c.writeWorkers.Start(c, c.writeWorkers.numWorkers)
}
