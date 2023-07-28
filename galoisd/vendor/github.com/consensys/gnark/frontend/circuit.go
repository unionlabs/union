/*
Copyright © 2020 ConsenSys Software Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package frontend

// Circuit must be implemented by user-defined circuit. The best way to define a
// circuit is to define a type which contains all the witness elements as fields
// and declare `Define` method on the type.
//
// For example, the following is a minimal valid circuit:
//
//	type MyCircuit struct {
//	    X frontend.Variable `gnark:"-,public"`
//	    Y frontend.Variable `gnark:"-,secret"`
//	}
//
//	func (c *MyCircuit) Define(api frontend.API) error {
//	    api.AssertIsEqual(c.X, c.Y)
//		return nil
//	}
//
// See the documentation for [schema.TagOpt] for how to use tags to define the
// behaviour of the compiler and schema parser.
type Circuit interface {
	// Define declares the circuit's Constraints
	Define(api API) error
}
