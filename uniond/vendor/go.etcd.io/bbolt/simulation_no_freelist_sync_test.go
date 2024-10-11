package bbolt_test

import (
	"testing"

	bolt "go.etcd.io/bbolt"
)

func TestSimulateNoFreeListSync_1op_1p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 1, 1)
}
func TestSimulateNoFreeListSync_10op_1p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10, 1)
}
func TestSimulateNoFreeListSync_100op_1p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 100, 1)
}
func TestSimulateNoFreeListSync_1000op_1p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 1000, 1)
}
func TestSimulateNoFreeListSync_10000op_1p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10000, 1)
}
func TestSimulateNoFreeListSync_10op_10p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10, 10)
}
func TestSimulateNoFreeListSync_100op_10p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 100, 10)
}
func TestSimulateNoFreeListSync_1000op_10p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 1000, 10)
}
func TestSimulateNoFreeListSync_10000op_10p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10000, 10)
}
func TestSimulateNoFreeListSync_100op_100p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 100, 100)
}
func TestSimulateNoFreeListSync_1000op_100p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 1000, 100)
}
func TestSimulateNoFreeListSync_10000op_100p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10000, 100)
}
func TestSimulateNoFreeListSync_10000op_1000p(t *testing.T) {
	testSimulate(t, &bolt.Options{NoFreelistSync: true}, 8, 10000, 1000)
}
