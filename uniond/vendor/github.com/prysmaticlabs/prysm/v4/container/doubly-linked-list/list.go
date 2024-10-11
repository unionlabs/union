package doublylinkedlist

import "github.com/pkg/errors"

var (
	errNextOnNil  = errors.New("cannot get next node of nil node")
	errPrevOnNil  = errors.New("cannot get previous node of nil node")
	errValueOnNil = errors.New("cannot get value of nil node")
)

// List is a generic doubly-linked list whose nodes can store any data type.
// It allows retrieving pointers to the first and last nodes.
type List[T any] struct {
	first *Node[T]
	last  *Node[T]
	len   int
}

// Node is a generic data structure that contains three fields:
// references to the previous and to the next node in the list, and one data field.
type Node[T any] struct {
	value T
	prev  *Node[T]
	next  *Node[T]
}

// Copy returns a copy of the origina list.
func (l *List[T]) Copy() *List[T] {
	if l == nil {
		return nil
	}
	list := &List[T]{}
	if l.len == 0 {
		return list
	}
	for n := l.First(); n != nil; n = n.next {
		list.Append(n.Copy())
	}
	return list
}

// First gets the reference to the first node in the list.
func (l *List[T]) First() *Node[T] {
	return l.first
}

// Last gets the reference to the last node in the list.
func (l *List[T]) Last() *Node[T] {
	return l.last
}

// Len gets the length of the list.
func (l *List[T]) Len() int {
	return l.len
}

// Append adds the passed in node to the end of the list.
func (l *List[T]) Append(n *Node[T]) {
	if l.first == nil {
		l.first = n
	} else {
		n.prev = l.last
		l.last.next = n
	}
	l.last = n
	l.len++
}

// Remove removes the passed in node from the list.
func (l *List[T]) Remove(n *Node[T]) {
	if n == nil {
		return
	}

	if n == l.First() {
		if n == l.last {
			l.first = nil
			l.last = nil
		} else {
			n.next.prev = nil
			l.first = n.next
		}
	} else {
		if n == l.last {
			n.prev.next = nil
			l.last = n.prev
		} else {
			if n.next == nil || n.prev == nil {
				// The node is not in the list,
				// otherwise it would be in the middle of two nodes.
				return
			}
			n.prev.next = n.next
			n.next.prev = n.prev
		}
	}
	l.len--
}

// NewNode creates a new node with the passed in value.
func NewNode[T any](value T) *Node[T] {
	return &Node[T]{value: value}
}

// Next gets the node's successor node.
func (n *Node[T]) Next() (*Node[T], error) {
	if n == nil {
		return nil, errNextOnNil
	}
	return n.next, nil
}

// Prev gets the node's predecessor node.
func (n *Node[T]) Prev() (*Node[T], error) {
	if n == nil {
		return nil, errPrevOnNil
	}
	return n.prev, nil
}

// Value gets the value stored in the node.
func (n *Node[T]) Value() (T, error) {
	if n == nil {
		return *new(T), errValueOnNil
	}
	return n.value, nil
}

// Copy copies the given node and returns a new one. It does not do a deep copy
// of T.
func (n *Node[T]) Copy() *Node[T] {
	if n == nil {
		return nil
	}
	return NewNode(n.value)
}
