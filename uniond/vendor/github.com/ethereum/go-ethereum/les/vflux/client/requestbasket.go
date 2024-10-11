// Copyright 2020 The go-ethereum Authors
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

package client

import (
	"io"

	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/rlp"
)

const basketFactor = 1000000 // reference basket amount and value scale factor

// referenceBasket keeps track of global request usage statistics and the usual prices
// of each used request type relative to each other. The amounts in the basket are scaled
// up by basketFactor because of the exponential expiration of long-term statistical data.
// Values are scaled so that the sum of all amounts and the sum of all values are equal.
//
// reqValues represent the internal relative value estimates for each request type and are
// calculated as value / amount. The average reqValue of all used requests is 1.
// In other words: SUM(refBasket[type].amount * reqValue[type]) = SUM(refBasket[type].amount)
type referenceBasket struct {
	basket    requestBasket
	reqValues []float64 // contents are read only, new slice is created for each update
}

// serverBasket collects served request amount and value statistics for a single server.
//
// Values are gradually transferred to the global reference basket with a long time
// constant so that each server basket represents long term usage and price statistics.
// When the transferred part is added to the reference basket the values are scaled so
// that their sum equals the total value calculated according to the previous reqValues.
// The ratio of request values coming from the server basket represent the pricing of
// the specific server and modify the global estimates with a weight proportional to
// the amount of service provided by the server.
type serverBasket struct {
	basket   requestBasket
	rvFactor float64
}

type (
	// requestBasket holds amounts and values for each request type.
	// These values are exponentially expired (see utils.ExpiredValue). The power of 2
	// exponent is applicable to all values within.
	requestBasket struct {
		items []basketItem
		exp   uint64
	}
	// basketItem holds amount and value for a single request type. Value is the total
	// relative request value accumulated for served requests while amount is the counter
	// for each request type.
	// Note that these values are both scaled up by basketFactor because of the exponential
	// expiration.
	basketItem struct {
		amount, value uint64
	}
)

// setExp sets the power of 2 exponent of the structure, scaling base values (the amounts
// and request values) up or down if necessary.
func (b *requestBasket) setExp(exp uint64) {
	if exp > b.exp {
		shift := exp - b.exp
		for i, item := range b.items {
			item.amount >>= shift
			item.value >>= shift
			b.items[i] = item
		}
		b.exp = exp
	}
	if exp < b.exp {
		shift := b.exp - exp
		for i, item := range b.items {
			item.amount <<= shift
			item.value <<= shift
			b.items[i] = item
		}
		b.exp = exp
	}
}

// init initializes a new server basket with the given service vector size (number of
// different request types)
func (s *serverBasket) init(size int) {
	if s.basket.items == nil {
		s.basket.items = make([]basketItem, size)
	}
}

// add adds the give type and amount of requests to the basket. Cost is calculated
// according to the server's own cost table.
func (s *serverBasket) add(reqType, reqAmount uint32, reqCost uint64, expFactor utils.ExpirationFactor) {
	s.basket.setExp(expFactor.Exp)
	i := &s.basket.items[reqType]
	i.amount += uint64(float64(uint64(reqAmount)*basketFactor) * expFactor.Factor)
	i.value += uint64(float64(reqCost) * s.rvFactor * expFactor.Factor)
}

// updateRvFactor updates the request value factor that scales server costs into the
// local value dimensions.
func (s *serverBasket) updateRvFactor(rvFactor float64) {
	s.rvFactor = rvFactor
}

// transfer decreases amounts and values in the basket with the given ratio and
// moves the removed amounts into a new basket which is returned and can be added
// to the global reference basket.
func (s *serverBasket) transfer(ratio float64) requestBasket {
	res := requestBasket{
		items: make([]basketItem, len(s.basket.items)),
		exp:   s.basket.exp,
	}
	for i, v := range s.basket.items {
		ta := uint64(float64(v.amount) * ratio)
		tv := uint64(float64(v.value) * ratio)
		if ta > v.amount {
			ta = v.amount
		}
		if tv > v.value {
			tv = v.value
		}
		s.basket.items[i] = basketItem{v.amount - ta, v.value - tv}
		res.items[i] = basketItem{ta, tv}
	}
	return res
}

// init initializes the reference basket with the given service vector size (number of
// different request types)
func (r *referenceBasket) init(size int) {
	r.reqValues = make([]float64, size)
	r.normalize()
	r.updateReqValues()
}

// add adds the transferred part of a server basket to the reference basket while scaling
// value amounts so that their sum equals the total value calculated according to the
// previous reqValues.
func (r *referenceBasket) add(newBasket requestBasket) {
	r.basket.setExp(newBasket.exp)
	// scale newBasket to match service unit value
	var (
		totalCost  uint64
		totalValue float64
	)
	for i, v := range newBasket.items {
		totalCost += v.value
		totalValue += float64(v.amount) * r.reqValues[i]
	}
	if totalCost > 0 {
		// add to reference with scaled values
		scaleValues := totalValue / float64(totalCost)
		for i, v := range newBasket.items {
			r.basket.items[i].amount += v.amount
			r.basket.items[i].value += uint64(float64(v.value) * scaleValues)
		}
	}
	r.updateReqValues()
}

// updateReqValues recalculates reqValues after adding transferred baskets. Note that
// values should be normalized first.
func (r *referenceBasket) updateReqValues() {
	r.reqValues = make([]float64, len(r.reqValues))
	for i, b := range r.basket.items {
		if b.amount > 0 {
			r.reqValues[i] = float64(b.value) / float64(b.amount)
		} else {
			r.reqValues[i] = 0
		}
	}
}

// normalize ensures that the sum of values equal the sum of amounts in the basket.
func (r *referenceBasket) normalize() {
	var sumAmount, sumValue uint64
	for _, b := range r.basket.items {
		sumAmount += b.amount
		sumValue += b.value
	}
	add := float64(int64(sumAmount-sumValue)) / float64(sumValue)
	for i, b := range r.basket.items {
		b.value += uint64(int64(float64(b.value) * add))
		r.basket.items[i] = b
	}
}

// reqValueFactor calculates the request value factor applicable to the server with
// the given announced request cost list
func (r *referenceBasket) reqValueFactor(costList []uint64) float64 {
	var (
		totalCost  float64
		totalValue uint64
	)
	for i, b := range r.basket.items {
		totalCost += float64(costList[i]) * float64(b.amount) // use floats to avoid overflow
		totalValue += b.value
	}
	if totalCost < 1 {
		return 0
	}
	return float64(totalValue) * basketFactor / totalCost
}

// EncodeRLP implements rlp.Encoder
func (b *basketItem) EncodeRLP(w io.Writer) error {
	return rlp.Encode(w, []interface{}{b.amount, b.value})
}

// DecodeRLP implements rlp.Decoder
func (b *basketItem) DecodeRLP(s *rlp.Stream) error {
	var item struct {
		Amount, Value uint64
	}
	if err := s.Decode(&item); err != nil {
		return err
	}
	b.amount, b.value = item.Amount, item.Value
	return nil
}

// EncodeRLP implements rlp.Encoder
func (r *requestBasket) EncodeRLP(w io.Writer) error {
	return rlp.Encode(w, []interface{}{r.items, r.exp})
}

// DecodeRLP implements rlp.Decoder
func (r *requestBasket) DecodeRLP(s *rlp.Stream) error {
	var enc struct {
		Items []basketItem
		Exp   uint64
	}
	if err := s.Decode(&enc); err != nil {
		return err
	}
	r.items, r.exp = enc.Items, enc.Exp
	return nil
}

// convertMapping converts a basket loaded from the database into the current format.
// If the available request types and their mapping into the service vector differ from
// the one used when saving the basket then this function reorders old fields and fills
// in previously unknown fields by scaling up amounts and values taken from the
// initialization basket.
func (r requestBasket) convertMapping(oldMapping, newMapping []string, initBasket requestBasket) requestBasket {
	nameMap := make(map[string]int)
	for i, name := range oldMapping {
		nameMap[name] = i
	}
	rc := requestBasket{items: make([]basketItem, len(newMapping))}
	var scale, oldScale, newScale float64
	for i, name := range newMapping {
		if ii, ok := nameMap[name]; ok {
			rc.items[i] = r.items[ii]
			oldScale += float64(initBasket.items[i].amount) * float64(initBasket.items[i].amount)
			newScale += float64(rc.items[i].amount) * float64(initBasket.items[i].amount)
		}
	}
	if oldScale > 1e-10 {
		scale = newScale / oldScale
	} else {
		scale = 1
	}
	for i, name := range newMapping {
		if _, ok := nameMap[name]; !ok {
			rc.items[i].amount = uint64(float64(initBasket.items[i].amount) * scale)
			rc.items[i].value = uint64(float64(initBasket.items[i].value) * scale)
		}
	}
	return rc
}
