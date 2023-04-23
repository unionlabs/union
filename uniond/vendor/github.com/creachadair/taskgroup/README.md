# taskgroup

[![GoDoc](https://img.shields.io/static/v1?label=godoc&message=reference&color=blue)](https://pkg.go.dev/github.com/creachadair/taskgroup)

A `*taskgroup.Group` represents a group of goroutines working on related tasks.
New tasks can be added to the group at will, and the caller can wait until all
tasks are complete. Errors are automatically collected and delivered to a
user-provided callback in a single goroutine.  This does not replace the full
generality of Go's built-in features, but it simplifies some of the plumbing
for common concurrent tasks.

Here is a [working example in the Go Playground](https://play.golang.org/p/V2slrnMu2Ec).

## Rationale

Go provides excellent and powerful concurrency primitives, in the form of
[goroutines](http://golang.org/ref/spec#Go_statements),
[channels](http://golang.org/ref/spec#Channel_types),
[select](http://golang.org/ref/spec#Select_statements), and the standard
library's [sync](http://godoc.org/sync) package. In some common situations,
however, managing goroutine lifetimes can become unwieldy using only what is
built in.

For example, consider the case of copying a large directory tree: Walk through
the source directory recursively, creating a parallel target directory
structure and starting a goroutine to copy each of the files concurrently.
That much is conceptually simple:

```go
func copyTree(source, target string) error {
	err := filepath.Walk(source, func(path string, fi os.FileInfo, err error) error {
		adjusted := adjustPath(path)
		if fi.IsDir() {
			return os.MkdirAll(adjusted, 0755)
		}
		go copyFile(adjusted, target)
		return nil
	})
	if err != nil {
		// ... clean up the output directory ...
	}
	return err
}
```

Unfortunately, it isn't quite sufficient: How will we detect when all the
file copies are finished? Typically we will use a `sync.WaitGroup`:

```go
var wg sync.WaitGroup
...
wg.Add(1)
go func() {
    defer wg.Done()
    copyFile(adjusted, target)
}()
...
wg.Wait() // blocks until all the tasks signal done
```

In addition, we need to cope with errors: Copies might fail (the disk might fill
up, or there might be a permissions error, say). For some applications it might
suffice to log the error and continue, but often in case of error you'd like
to back out and clean up your mess. To do this, we need to capture the return
value from the function inside the goroutine―and that will require us either to
add a lock or plumb in another channel:

```go
errs := make(chan error)
...
go copyFile(adjusted, target, errs)
```

Since multiple copies can be running in parallel, we will also need another
goroutine to drain the errors channel and accumulate the results somewhere:

```go
var failures []error
go func() {
    for e := range errs {
        failures = append(failures, e)
    }
}()
...
wg.Wait()
close(errs)
```

But now we need to also detect when the error collector is finished, so that
we can examine the `failures` without a data race. We'll need another channel
or wait group to signal for this:

```go
var failures []error
edone := make(chan struct{})
go func() {
    defer close(edone)
    for e := range errs {
        failures = append(failures, e)
	}
}()
...
wg.Wait()   // all the workers are done
close(errs) // signal the error collector to stop
<-edone     // wait for the error collector to be done
```

All of this is tedious, but works fine. Now, however, if something fails we
really don't want to wait around for all the copies to finish―if one of the
file copies fails, we want to stop what you're doing and clean up.  So now we
need a way to signal cancellation, such as a `context.Context`:

```go
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	...
	copyFile(ctx, adjusted, target, errs)
```

and then `copyFile` will have to check for that:

```go
func copyFile(ctx context.Context, source, target string, errs chan<- error) {
	if ctx.Err() != nil {
		return
	}
 	// ... do the copy as normal, or propagate an error
}
```

Finally, we need some way to limit the number of concurrent copies. Even if the
host has plenty of memory and CPU, unbounded concurrency is likely to run us out of
file descriptors. As a rule of thumb, unbounded concurrency is almost always a poor
choice. We might use a [semaphore](https://godoc.org/golang.org/x/sync/semaphore) or
a throttling channel:

```go
throttle := make(chan struct{}, 64)  // allow up to 64 concurrent copies
go func() {
   throttle <- struct{}{} // block until the throttle has a free slot
   defer func() { wg.Done(); <-throttle }()
   copyFile(ctx, adjusted, target, errs)
}()
```

In case you've lost count, we're up to four channels (including the context).

The point here is that, while Go's concurrency primitives are more than
powerful enough to express these relationships, it can be tedious to wire them
all together.

The `taskgroup` package exists to handle the plumbing for the common case of a
group of tasks that are all working on a related outcome (_e.g.,_ copying a
directory structure), and where an error on the part of any _single_ task may
be grounds for terminating the work as a whole.

The package provides a `taskgroup.Group` type that has built-in support for
some of these concerns:

- Limiting the number of active goroutines.
- Collecting and filtering errors.
- Waiting for completion and delivering status.

A `taskgroup.Group` collects error values from each task and can deliver them
to a user-provided callback. The callback can filter them or take other actions
(such as cancellation). Invocations of the callback are all done from a single
goroutine so it is safe to manipulate local resources without a lock.

A group does not directly support cancellation, but integrates cleanly with the
standard [context](https://godoc.org/context) package. A `context.CancelFunc`
can be used as a trigger to signal the whole group when an error occurs.

The API is straightforward: A task is expressed as a `func() error`, and is
added to a group using the `Go` method,

```go
g := taskgroup.New(nil).Go(myTask)
```

Any number of tasks may be added, and it is safe to do so from multiple
goroutines concurrently.  To wait for the tasks to finish, use:

```go
err := g.Wait()
```

This blocks until all the tasks in the group have returned (either
successfully, or with an error). `Wait` returns the first non-nil error
returned by any of the worker tasks.

An example program can be found in `examples/copytree/copytree.go`.

## Filtering Errors

The `taskgroup.New` function takes an optional callback that is invoked for
each non-nil error reported by a task in the group. The callback may choose
to propagate, replace, or discard the error. For example:

```go
g := taskgroup.New(func(err error) error {
   if os.IsNotExist(err) {
      return nil // ignore files that do not exist
   }
   return err
})
```

## Controlling Concurrency

The `Limit` method supports limiting the number of concurrently _active_
goroutines in the group. For example:

```go
// Allow at most 25 concurrently-active goroutines in the group.
g, start := taskgroup.New(nil).Limit(25)

// Start tasks by calling the function returned by taskgroup.Limit:
start(task1)
start(task2)
// ...
```

## Solo Tasks

In some cases it is useful to have a solo background task to handle a separate
concern, such as collecting the results from a batch of concurrent workers.

For example, suppose we have a group of tasks processing a complex search, and
we want to aggregate the results.  We can't do this from the same group that
the workers are using, since it needs to run until they have all completed.  On
the other hand, creating two full groups is overkill, since we only need one
additional goroutine.

To handle such cases, the `Single` constructor is helpful: It manages a single
background goroutine with a separate wait:

```go
var total int
results := make(chan int)
s := taskgroup.Single(func() error {
    for v := range results {
        total += v
    }
})

g := taskgroup.New(nil)
for i := 0; i < numTasks; i++ {
    batch := newBatch(i)
    g.Go(func() error {
        return search(batch)
    })
}

// Wait for the workers to finish.
g.Wait()

// Signal the collector to stop, and wait for it to do so.
close(results)
s.Wait()

// Now it is safe to use the results.
fmt.Println(total)
```
