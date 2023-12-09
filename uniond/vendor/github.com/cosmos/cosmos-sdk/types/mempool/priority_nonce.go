package mempool

import (
	"context"
	"fmt"
	"math"
	"sync"

	"github.com/huandu/skiplist"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

var (
	_ Mempool  = (*PriorityNonceMempool[int64])(nil)
	_ Iterator = (*PriorityNonceIterator[int64])(nil)
)

type (
	// PriorityNonceMempoolConfig defines the configuration used to configure the
	// PriorityNonceMempool.
	PriorityNonceMempoolConfig[C comparable] struct {
		// TxPriority defines the transaction priority and comparator.
		TxPriority TxPriority[C]

		// OnRead is a callback to be called when a tx is read from the mempool.
		OnRead func(tx sdk.Tx)

		// TxReplacement is a callback to be called when duplicated transaction nonce
		// detected during mempool insert. An application can define a transaction
		// replacement rule based on tx priority or certain transaction fields.
		TxReplacement func(op, np C, oTx, nTx sdk.Tx) bool

		// MaxTx sets the maximum number of transactions allowed in the mempool with
		// the semantics:
		// - if MaxTx == 0, there is no cap on the number of transactions in the mempool
		// - if MaxTx > 0, the mempool will cap the number of transactions it stores,
		//   and will prioritize transactions by their priority and sender-nonce
		//   (sequence number) when evicting transactions.
		// - if MaxTx < 0, `Insert` is a no-op.
		MaxTx int
	}

	// PriorityNonceMempool is a mempool implementation that stores txs
	// in a partially ordered set by 2 dimensions: priority, and sender-nonce
	// (sequence number). Internally it uses one priority ordered skip list and one
	// skip list per sender ordered by sender-nonce (sequence number). When there
	// are multiple txs from the same sender, they are not always comparable by
	// priority to other sender txs and must be partially ordered by both sender-nonce
	// and priority.
	PriorityNonceMempool[C comparable] struct {
		mtx            sync.Mutex
		priorityIndex  *skiplist.SkipList
		priorityCounts map[C]int
		senderIndices  map[string]*skiplist.SkipList
		scores         map[txMeta[C]]txMeta[C]
		cfg            PriorityNonceMempoolConfig[C]
	}

	// PriorityNonceIterator defines an iterator that is used for mempool iteration
	// on Select().
	PriorityNonceIterator[C comparable] struct {
		mempool       *PriorityNonceMempool[C]
		priorityNode  *skiplist.Element
		senderCursors map[string]*skiplist.Element
		sender        string
		nextPriority  C
	}

	// TxPriority defines a type that is used to retrieve and compare transaction
	// priorities. Priorities must be comparable.
	TxPriority[C comparable] struct {
		// GetTxPriority returns the priority of the transaction. A priority must be
		// comparable via Compare.
		GetTxPriority func(ctx context.Context, tx sdk.Tx) C

		// Compare compares two transaction priorities. The result must be 0 if
		// a == b, -1 if a < b, and +1 if a > b.
		Compare func(a, b C) int

		// MinValue defines the minimum priority value, e.g. MinInt64. This value is
		// used when instantiating a new iterator and comparing weights.
		MinValue C
	}

	// txMeta stores transaction metadata used in indices
	txMeta[C comparable] struct {
		// nonce is the sender's sequence number
		nonce uint64
		// priority is the transaction's priority
		priority C
		// sender is the transaction's sender
		sender string
		// weight is the transaction's weight, used as a tiebreaker for transactions
		// with the same priority
		weight C
		// senderElement is a pointer to the transaction's element in the sender index
		senderElement *skiplist.Element
	}
)

// NewDefaultTxPriority returns a TxPriority comparator using ctx.Priority as
// the defining transaction priority.
func NewDefaultTxPriority() TxPriority[int64] {
	return TxPriority[int64]{
		GetTxPriority: func(goCtx context.Context, _ sdk.Tx) int64 {
			return sdk.UnwrapSDKContext(goCtx).Priority()
		},
		Compare: func(a, b int64) int {
			return skiplist.Int64.Compare(a, b)
		},
		MinValue: math.MinInt64,
	}
}

func DefaultPriorityNonceMempoolConfig() PriorityNonceMempoolConfig[int64] {
	return PriorityNonceMempoolConfig[int64]{
		TxPriority: NewDefaultTxPriority(),
	}
}

// skiplistComparable is a comparator for txKeys that first compares priority,
// then weight, then sender, then nonce, uniquely identifying a transaction.
//
// Note, skiplistComparable is used as the comparator in the priority index.
func skiplistComparable[C comparable](txPriority TxPriority[C]) skiplist.Comparable {
	return skiplist.LessThanFunc(func(a, b any) int {
		keyA := a.(txMeta[C])
		keyB := b.(txMeta[C])

		res := txPriority.Compare(keyA.priority, keyB.priority)
		if res != 0 {
			return res
		}

		// Weight is used as a tiebreaker for transactions with the same priority.
		// Weight is calculated in a single pass in .Select(...) and so will be 0
		// on .Insert(...).
		res = txPriority.Compare(keyA.weight, keyB.weight)
		if res != 0 {
			return res
		}

		// Because weight will be 0 on .Insert(...), we must also compare sender and
		// nonce to resolve priority collisions. If we didn't then transactions with
		// the same priority would overwrite each other in the priority index.
		res = skiplist.String.Compare(keyA.sender, keyB.sender)
		if res != 0 {
			return res
		}

		return skiplist.Uint64.Compare(keyA.nonce, keyB.nonce)
	})
}

// NewPriorityMempool returns the SDK's default mempool implementation which
// returns txs in a partial order by 2 dimensions; priority, and sender-nonce.
func NewPriorityMempool[C comparable](cfg PriorityNonceMempoolConfig[C]) *PriorityNonceMempool[C] {
	mp := &PriorityNonceMempool[C]{
		priorityIndex:  skiplist.New(skiplistComparable(cfg.TxPriority)),
		priorityCounts: make(map[C]int),
		senderIndices:  make(map[string]*skiplist.SkipList),
		scores:         make(map[txMeta[C]]txMeta[C]),
		cfg:            cfg,
	}

	return mp
}

// DefaultPriorityMempool returns a priorityNonceMempool with no options.
func DefaultPriorityMempool() *PriorityNonceMempool[int64] {
	return NewPriorityMempool(DefaultPriorityNonceMempoolConfig())
}

// NextSenderTx returns the next transaction for a given sender by nonce order,
// i.e. the next valid transaction for the sender. If no such transaction exists,
// nil will be returned.
func (mp *PriorityNonceMempool[C]) NextSenderTx(sender string) sdk.Tx {
	senderIndex, ok := mp.senderIndices[sender]
	if !ok {
		return nil
	}

	cursor := senderIndex.Front()
	return cursor.Value.(sdk.Tx)
}

// Insert attempts to insert a Tx into the app-side mempool in O(log n) time,
// returning an error if unsuccessful. Sender and nonce are derived from the
// transaction's first signature.
//
// Transactions are unique by sender and nonce. Inserting a duplicate tx is an
// O(log n) no-op.
//
// Inserting a duplicate tx with a different priority overwrites the existing tx,
// changing the total order of the mempool.
func (mp *PriorityNonceMempool[C]) Insert(ctx context.Context, tx sdk.Tx) error {
	mp.mtx.Lock()
	defer mp.mtx.Unlock()
	if mp.cfg.MaxTx > 0 && mp.priorityIndex.Len() >= mp.cfg.MaxTx {
		return ErrMempoolTxMaxCapacity
	} else if mp.cfg.MaxTx < 0 {
		return nil
	}

	sigs, err := tx.(signing.SigVerifiableTx).GetSignaturesV2()
	if err != nil {
		return err
	}
	if len(sigs) == 0 {
		return fmt.Errorf("tx must have at least one signer")
	}

	sig := sigs[0]
	sender := sdk.AccAddress(sig.PubKey.Address()).String()
	priority := mp.cfg.TxPriority.GetTxPriority(ctx, tx)
	nonce := sig.Sequence
	key := txMeta[C]{nonce: nonce, priority: priority, sender: sender}

	senderIndex, ok := mp.senderIndices[sender]
	if !ok {
		senderIndex = skiplist.New(skiplist.LessThanFunc(func(a, b any) int {
			return skiplist.Uint64.Compare(b.(txMeta[C]).nonce, a.(txMeta[C]).nonce)
		}))

		// initialize sender index if not found
		mp.senderIndices[sender] = senderIndex
	}

	// Since mp.priorityIndex is scored by priority, then sender, then nonce, a
	// changed priority will create a new key, so we must remove the old key and
	// re-insert it to avoid having the same tx with different priorityIndex indexed
	// twice in the mempool.
	//
	// This O(log n) remove operation is rare and only happens when a tx's priority
	// changes.
	sk := txMeta[C]{nonce: nonce, sender: sender}
	if oldScore, txExists := mp.scores[sk]; txExists {
		if mp.cfg.TxReplacement != nil && !mp.cfg.TxReplacement(oldScore.priority, priority, senderIndex.Get(key).Value.(sdk.Tx), tx) {
			return fmt.Errorf(
				"tx doesn't fit the replacement rule, oldPriority: %v, newPriority: %v, oldTx: %v, newTx: %v",
				oldScore.priority,
				priority,
				senderIndex.Get(key).Value.(sdk.Tx),
				tx,
			)
		}

		mp.priorityIndex.Remove(txMeta[C]{
			nonce:    nonce,
			sender:   sender,
			priority: oldScore.priority,
			weight:   oldScore.weight,
		})
		mp.priorityCounts[oldScore.priority]--
	}

	mp.priorityCounts[priority]++

	// Since senderIndex is scored by nonce, a changed priority will overwrite the
	// existing key.
	key.senderElement = senderIndex.Set(key, tx)

	mp.scores[sk] = txMeta[C]{priority: priority}
	mp.priorityIndex.Set(key, tx)

	return nil
}

func (i *PriorityNonceIterator[C]) iteratePriority() Iterator {
	// beginning of priority iteration
	if i.priorityNode == nil {
		i.priorityNode = i.mempool.priorityIndex.Front()
	} else {
		i.priorityNode = i.priorityNode.Next()
	}

	// end of priority iteration
	if i.priorityNode == nil {
		return nil
	}

	i.sender = i.priorityNode.Key().(txMeta[C]).sender

	nextPriorityNode := i.priorityNode.Next()
	if nextPriorityNode != nil {
		i.nextPriority = nextPriorityNode.Key().(txMeta[C]).priority
	} else {
		i.nextPriority = i.mempool.cfg.TxPriority.MinValue
	}

	return i.Next()
}

func (i *PriorityNonceIterator[C]) Next() Iterator {
	if i.priorityNode == nil {
		return nil
	}

	cursor, ok := i.senderCursors[i.sender]
	if !ok {
		// beginning of sender iteration
		cursor = i.mempool.senderIndices[i.sender].Front()
	} else {
		// middle of sender iteration
		cursor = cursor.Next()
	}

	// end of sender iteration
	if cursor == nil {
		return i.iteratePriority()
	}

	key := cursor.Key().(txMeta[C])

	// We've reached a transaction with a priority lower than the next highest
	// priority in the pool.
	if i.mempool.cfg.TxPriority.Compare(key.priority, i.nextPriority) < 0 {
		return i.iteratePriority()
	} else if i.priorityNode.Next() != nil && i.mempool.cfg.TxPriority.Compare(key.priority, i.nextPriority) == 0 {
		// Weight is incorporated into the priority index key only (not sender index)
		// so we must fetch it here from the scores map.
		weight := i.mempool.scores[txMeta[C]{nonce: key.nonce, sender: key.sender}].weight
		if i.mempool.cfg.TxPriority.Compare(weight, i.priorityNode.Next().Key().(txMeta[C]).weight) < 0 {
			return i.iteratePriority()
		}
	}

	i.senderCursors[i.sender] = cursor
	return i
}

func (i *PriorityNonceIterator[C]) Tx() sdk.Tx {
	return i.senderCursors[i.sender].Value.(sdk.Tx)
}

// Select returns a set of transactions from the mempool, ordered by priority
// and sender-nonce in O(n) time. The passed in list of transactions are ignored.
// This is a readonly operation, the mempool is not modified.
//
// The maxBytes parameter defines the maximum number of bytes of transactions to
// return.
//
// NOTE: It is not safe to use this iterator while removing transactions from
// the underlying mempool.
func (mp *PriorityNonceMempool[C]) Select(_ context.Context, _ [][]byte) Iterator {
	mp.mtx.Lock()
	defer mp.mtx.Unlock()
	if mp.priorityIndex.Len() == 0 {
		return nil
	}

	mp.reorderPriorityTies()

	iterator := &PriorityNonceIterator[C]{
		mempool:       mp,
		senderCursors: make(map[string]*skiplist.Element),
	}

	return iterator.iteratePriority()
}

type reorderKey[C comparable] struct {
	deleteKey txMeta[C]
	insertKey txMeta[C]
	tx        sdk.Tx
}

func (mp *PriorityNonceMempool[C]) reorderPriorityTies() {
	node := mp.priorityIndex.Front()

	var reordering []reorderKey[C]
	for node != nil {
		key := node.Key().(txMeta[C])
		if mp.priorityCounts[key.priority] > 1 {
			newKey := key
			newKey.weight = senderWeight(mp.cfg.TxPriority, key.senderElement)
			reordering = append(reordering, reorderKey[C]{deleteKey: key, insertKey: newKey, tx: node.Value.(sdk.Tx)})
		}

		node = node.Next()
	}

	for _, k := range reordering {
		mp.priorityIndex.Remove(k.deleteKey)
		delete(mp.scores, txMeta[C]{nonce: k.deleteKey.nonce, sender: k.deleteKey.sender})
		mp.priorityIndex.Set(k.insertKey, k.tx)
		mp.scores[txMeta[C]{nonce: k.insertKey.nonce, sender: k.insertKey.sender}] = k.insertKey
	}
}

// senderWeight returns the weight of a given tx (t) at senderCursor. Weight is
// defined as the first (nonce-wise) same sender tx with a priority not equal to
// t. It is used to resolve priority collisions, that is when 2 or more txs from
// different senders have the same priority.
func senderWeight[C comparable](txPriority TxPriority[C], senderCursor *skiplist.Element) C {
	if senderCursor == nil {
		return txPriority.MinValue
	}

	weight := senderCursor.Key().(txMeta[C]).priority
	senderCursor = senderCursor.Next()
	for senderCursor != nil {
		p := senderCursor.Key().(txMeta[C]).priority
		if txPriority.Compare(p, weight) != 0 {
			weight = p
		}

		senderCursor = senderCursor.Next()
	}

	return weight
}

// CountTx returns the number of transactions in the mempool.
func (mp *PriorityNonceMempool[C]) CountTx() int {
	mp.mtx.Lock()
	defer mp.mtx.Unlock()
	return mp.priorityIndex.Len()
}

// Remove removes a transaction from the mempool in O(log n) time, returning an
// error if unsuccessful.
func (mp *PriorityNonceMempool[C]) Remove(tx sdk.Tx) error {
	mp.mtx.Lock()
	defer mp.mtx.Unlock()
	sigs, err := tx.(signing.SigVerifiableTx).GetSignaturesV2()
	if err != nil {
		return err
	}
	if len(sigs) == 0 {
		return fmt.Errorf("attempted to remove a tx with no signatures")
	}

	sig := sigs[0]
	sender := sdk.AccAddress(sig.PubKey.Address()).String()
	nonce := sig.Sequence

	scoreKey := txMeta[C]{nonce: nonce, sender: sender}
	score, ok := mp.scores[scoreKey]
	if !ok {
		return ErrTxNotFound
	}
	tk := txMeta[C]{nonce: nonce, priority: score.priority, sender: sender, weight: score.weight}

	senderTxs, ok := mp.senderIndices[sender]
	if !ok {
		return fmt.Errorf("sender %s not found", sender)
	}

	mp.priorityIndex.Remove(tk)
	senderTxs.Remove(tk)
	delete(mp.scores, scoreKey)
	mp.priorityCounts[score.priority]--

	return nil
}

func IsEmpty[C comparable](mempool Mempool) error {
	mp := mempool.(*PriorityNonceMempool[C])
	if mp.priorityIndex.Len() != 0 {
		return fmt.Errorf("priorityIndex not empty")
	}

	countKeys := make([]C, 0, len(mp.priorityCounts))
	for k := range mp.priorityCounts {
		countKeys = append(countKeys, k)
	}

	for _, k := range countKeys {
		if mp.priorityCounts[k] != 0 {
			return fmt.Errorf("priorityCounts not zero at %v, got %v", k, mp.priorityCounts[k])
		}
	}

	senderKeys := make([]string, 0, len(mp.senderIndices))
	for k := range mp.senderIndices {
		senderKeys = append(senderKeys, k)
	}

	for _, k := range senderKeys {
		if mp.senderIndices[k].Len() != 0 {
			return fmt.Errorf("senderIndex not empty for sender %v", k)
		}
	}

	return nil
}
