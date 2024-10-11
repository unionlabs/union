// Copyright 2023 The go-ethereum Authors
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

package event

// JoinSubscriptions joins multiple subscriptions to be able to track them as
// one entity and collectively cancel them of consume any errors from them.
func JoinSubscriptions(subs ...Subscription) Subscription {
	return NewSubscription(func(unsubbed <-chan struct{}) error {
		// Unsubscribe all subscriptions before returning
		defer func() {
			for _, sub := range subs {
				sub.Unsubscribe()
			}
		}()
		// Wait for an error on any of the subscriptions and propagate up
		errc := make(chan error, len(subs))
		for i := range subs {
			go func(sub Subscription) {
				select {
				case err := <-sub.Err():
					if err != nil {
						errc <- err
					}
				case <-unsubbed:
				}
			}(subs[i])
		}

		select {
		case err := <-errc:
			return err
		case <-unsubbed:
			return nil
		}
	})
}
