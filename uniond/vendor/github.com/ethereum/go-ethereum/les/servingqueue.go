// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package les

import (
	"sync"
	"sync/atomic"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/common/prque"
	"golang.org/x/exp/slices"
)

// servingQueue allows running tasks in a limited number of threads and puts the
// waiting tasks in a priority queue
type servingQueue struct {
	recentTime, queuedTime     uint64
	servingTimeDiff            atomic.Uint64
	burstLimit, burstDropLimit uint64
	burstDecRate               float64
	lastUpdate                 mclock.AbsTime

	queueAddCh, queueBestCh chan *servingTask
	stopThreadCh, quit      chan struct{}
	setThreadsCh            chan int

	wg          sync.WaitGroup
	threadCount int                               // number of currently running threads
	queue       *prque.Prque[int64, *servingTask] // priority queue for waiting or suspended tasks
	best        *servingTask                      // the highest priority task (not included in the queue)
	suspendBias int64                             // priority bias against suspending an already running task
}

// servingTask represents a request serving task. Tasks can be implemented to
// run in multiple steps, allowing the serving queue to suspend execution between
// steps if higher priority tasks are entered. The creator of the task should
// set the following fields:
//
// - priority: greater value means higher priority; values can wrap around the int64 range
// - run: execute a single step; return true if finished
// - after: executed after run finishes or returns an error, receives the total serving time
type servingTask struct {
	sq                                       *servingQueue
	servingTime, timeAdded, maxTime, expTime uint64
	peer                                     *clientPeer
	priority                                 int64
	biasAdded                                bool
	token                                    runToken
	tokenCh                                  chan runToken
}

// runToken received by servingTask.start allows the task to run. Closing the
// channel by servingTask.stop signals the thread controller to allow a new task
// to start running.
type runToken chan struct{}

// start blocks until the task can start and returns true if it is allowed to run.
// Returning false means that the task should be cancelled.
func (t *servingTask) start() bool {
	if t.peer.isFrozen() {
		return false
	}
	t.tokenCh = make(chan runToken, 1)
	select {
	case t.sq.queueAddCh <- t:
	case <-t.sq.quit:
		return false
	}
	select {
	case t.token = <-t.tokenCh:
	case <-t.sq.quit:
		return false
	}
	if t.token == nil {
		return false
	}
	t.servingTime -= uint64(mclock.Now())
	return true
}

// done signals the thread controller about the task being finished and returns
// the total serving time of the task in nanoseconds.
func (t *servingTask) done() uint64 {
	t.servingTime += uint64(mclock.Now())
	close(t.token)
	diff := t.servingTime - t.timeAdded
	t.timeAdded = t.servingTime
	if t.expTime > diff {
		t.expTime -= diff
		t.sq.servingTimeDiff.Add(t.expTime)
	} else {
		t.expTime = 0
	}
	return t.servingTime
}

// waitOrStop can be called during the execution of the task. It blocks if there
// is a higher priority task waiting (a bias is applied in favor of the currently
// running task). Returning true means that the execution can be resumed. False
// means the task should be cancelled.
func (t *servingTask) waitOrStop() bool {
	t.done()
	if !t.biasAdded {
		t.priority += t.sq.suspendBias
		t.biasAdded = true
	}
	return t.start()
}

// newServingQueue returns a new servingQueue
func newServingQueue(suspendBias int64, utilTarget float64) *servingQueue {
	sq := &servingQueue{
		queue:          prque.New[int64, *servingTask](nil),
		suspendBias:    suspendBias,
		queueAddCh:     make(chan *servingTask, 100),
		queueBestCh:    make(chan *servingTask),
		stopThreadCh:   make(chan struct{}),
		quit:           make(chan struct{}),
		setThreadsCh:   make(chan int, 10),
		burstLimit:     uint64(utilTarget * bufLimitRatio * 1200000),
		burstDropLimit: uint64(utilTarget * bufLimitRatio * 1000000),
		burstDecRate:   utilTarget,
		lastUpdate:     mclock.Now(),
	}
	sq.wg.Add(2)
	go sq.queueLoop()
	go sq.threadCountLoop()
	return sq
}

// newTask creates a new task with the given priority
func (sq *servingQueue) newTask(peer *clientPeer, maxTime uint64, priority int64) *servingTask {
	return &servingTask{
		sq:       sq,
		peer:     peer,
		maxTime:  maxTime,
		expTime:  maxTime,
		priority: priority,
	}
}

// threadController is started in multiple goroutines and controls the execution
// of tasks. The number of active thread controllers equals the allowed number of
// concurrently running threads. It tries to fetch the highest priority queued
// task first. If there are no queued tasks waiting then it can directly catch
// run tokens from the token channel and allow the corresponding tasks to run
// without entering the priority queue.
func (sq *servingQueue) threadController() {
	defer sq.wg.Done()
	for {
		token := make(runToken)
		select {
		case best := <-sq.queueBestCh:
			best.tokenCh <- token
		case <-sq.stopThreadCh:
			return
		case <-sq.quit:
			return
		}
		select {
		case <-sq.stopThreadCh:
			return
		case <-sq.quit:
			return
		case <-token:
		}
	}
}

// peerTasks lists the tasks received from a given peer when selecting peers to freeze
type peerTasks struct {
	peer     *clientPeer
	list     []*servingTask
	sumTime  uint64
	priority float64
}

// freezePeers selects the peers with the worst priority queued tasks and freezes
// them until burstTime goes under burstDropLimit or all peers are frozen
func (sq *servingQueue) freezePeers() {
	peerMap := make(map[*clientPeer]*peerTasks)
	var peerList []*peerTasks
	if sq.best != nil {
		sq.queue.Push(sq.best, sq.best.priority)
	}
	sq.best = nil
	for sq.queue.Size() > 0 {
		task := sq.queue.PopItem()
		tasks := peerMap[task.peer]
		if tasks == nil {
			bufValue, bufLimit := task.peer.fcClient.BufferStatus()
			if bufLimit < 1 {
				bufLimit = 1
			}
			tasks = &peerTasks{
				peer:     task.peer,
				priority: float64(bufValue) / float64(bufLimit), // lower value comes first
			}
			peerMap[task.peer] = tasks
			peerList = append(peerList, tasks)
		}
		tasks.list = append(tasks.list, task)
		tasks.sumTime += task.expTime
	}
	slices.SortFunc(peerList, func(a, b *peerTasks) int {
		if a.priority < b.priority {
			return -1
		}
		if a.priority > b.priority {
			return 1
		}
		return 0
	})
	drop := true
	for _, tasks := range peerList {
		if drop {
			tasks.peer.freeze()
			tasks.peer.fcClient.Freeze()
			sq.queuedTime -= tasks.sumTime
			sqQueuedGauge.Update(int64(sq.queuedTime))
			clientFreezeMeter.Mark(1)
			drop = sq.recentTime+sq.queuedTime > sq.burstDropLimit
			for _, task := range tasks.list {
				task.tokenCh <- nil
			}
		} else {
			for _, task := range tasks.list {
				sq.queue.Push(task, task.priority)
			}
		}
	}
	if sq.queue.Size() > 0 {
		sq.best = sq.queue.PopItem()
	}
}

// updateRecentTime recalculates the recent serving time value
func (sq *servingQueue) updateRecentTime() {
	subTime := sq.servingTimeDiff.Swap(0)
	now := mclock.Now()
	dt := now - sq.lastUpdate
	sq.lastUpdate = now
	if dt > 0 {
		subTime += uint64(float64(dt) * sq.burstDecRate)
	}
	if sq.recentTime > subTime {
		sq.recentTime -= subTime
	} else {
		sq.recentTime = 0
	}
}

// addTask inserts a task into the priority queue
func (sq *servingQueue) addTask(task *servingTask) {
	if sq.best == nil {
		sq.best = task
	} else if task.priority-sq.best.priority > 0 {
		sq.queue.Push(sq.best, sq.best.priority)
		sq.best = task
	} else {
		sq.queue.Push(task, task.priority)
	}
	sq.updateRecentTime()
	sq.queuedTime += task.expTime
	sqServedGauge.Update(int64(sq.recentTime))
	sqQueuedGauge.Update(int64(sq.queuedTime))
	if sq.recentTime+sq.queuedTime > sq.burstLimit {
		sq.freezePeers()
	}
}

// queueLoop is an event loop running in a goroutine. It receives tasks from queueAddCh
// and always tries to send the highest priority task to queueBestCh. Successfully sent
// tasks are removed from the queue.
func (sq *servingQueue) queueLoop() {
	defer sq.wg.Done()
	for {
		if sq.best != nil {
			expTime := sq.best.expTime
			select {
			case task := <-sq.queueAddCh:
				sq.addTask(task)
			case sq.queueBestCh <- sq.best:
				sq.updateRecentTime()
				sq.queuedTime -= expTime
				sq.recentTime += expTime
				sqServedGauge.Update(int64(sq.recentTime))
				sqQueuedGauge.Update(int64(sq.queuedTime))
				if sq.queue.Size() == 0 {
					sq.best = nil
				} else {
					sq.best = sq.queue.PopItem()
				}
			case <-sq.quit:
				return
			}
		} else {
			select {
			case task := <-sq.queueAddCh:
				sq.addTask(task)
			case <-sq.quit:
				return
			}
		}
	}
}

// threadCountLoop is an event loop running in a goroutine. It adjusts the number
// of active thread controller goroutines.
func (sq *servingQueue) threadCountLoop() {
	var threadCountTarget int
	defer sq.wg.Done()
	for {
		for threadCountTarget > sq.threadCount {
			sq.wg.Add(1)
			go sq.threadController()
			sq.threadCount++
		}
		if threadCountTarget < sq.threadCount {
			select {
			case threadCountTarget = <-sq.setThreadsCh:
			case sq.stopThreadCh <- struct{}{}:
				sq.threadCount--
			case <-sq.quit:
				return
			}
		} else {
			select {
			case threadCountTarget = <-sq.setThreadsCh:
			case <-sq.quit:
				return
			}
		}
	}
}

// setThreads sets the allowed processing thread count, suspending tasks as soon as
// possible if necessary.
func (sq *servingQueue) setThreads(threadCount int) {
	select {
	case sq.setThreadsCh <- threadCount:
	case <-sq.quit:
		return
	}
}

// stop stops task processing as soon as possible and shuts down the serving queue.
func (sq *servingQueue) stop() {
	close(sq.quit)
	sq.wg.Wait()
}
