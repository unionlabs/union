// nolint:dupword
//
// Package slasher defines an optimized implementation of Ethereum proof-of-stake slashing
// detection, namely focused on catching "surround vote" slashable
// offenses as explained here: https://blog.ethereum.org/2020/01/13/validated-staking-on-eth2-1-incentives/.
//
// Surround vote detection is a difficult problem if done naively, as slasher
// needs to keep track of every single attestation by every single validator
// in the network and be ready to efficiently detect whether incoming attestations
// are slashable with respect to older ones. To do this, the Sigma Prime team
// created an elaborate design document: https://hackmd.io/@sproul/min-max-slasher
// offering an optimal solution.
//
// Attesting histories are kept for each validator in two separate arrays known
// as min and max spans, which are explained in our design document:
// https://hackmd.io/@prysmaticlabs/slasher.
//
// A regular pair of min and max spans for a validator look as follows
// with length = H where H is the amount of epochs worth of history
// we want to persist for slashing detection.
//
//	validator_1_min_span = [2, 2, 2, ..., 2]
//	validator_1_max_span = [0, 0, 0, ..., 0]
//
// Instead of always dealing with length H arrays, which can be prohibitively
// expensive to handle in memory, we split these arrays into chunks of length C.
// For C = 3, for example, the 0th chunk of validator 1's min and max spans would look
// as follows:
//
//	validator_1_min_span_chunk_0 = [2, 2, 2]
//	validator_1_max_span_chunk_0 = [2, 2, 2]
//
// Next, on disk, we take chunks for K validators, and store them as flat slices.
// For example, if H = 3, C = 3, and K = 3, then we can store 3 validators' chunks as a flat
// slice as follows:
//
//	   val0     val1     val2
//	    |        |        |
//	 {     }  {     }  {     }
//	[2, 2, 2, 2, 2, 2, 2, 2, 2]
//
// This is known as 2D chunking, pioneered by the Sigma Prime team here:
// https://hackmd.io/@sproul/min-max-slasher. The parameters H, C, and K will be
// used extensively throughout this package.
package slasher
