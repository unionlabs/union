package plonk

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

func UnmarshalSolidity(s []byte, nbCommits int) Proof {

	var proof Proof
	offset := 0
	point_size := 64
	fr_size := 32
	proof.BatchedProof.ClaimedValues = make([]fr.Element, 7+nbCommits)
	proof.Bsb22Commitments = make([]bn254.G1Affine, nbCommits)

	// uint256 l_com_x;
	// uint256 l_com_y;
	// uint256 r_com_x;
	// uint256 r_com_y;
	// uint256 o_com_x;
	// uint256 o_com_y;
	for i := 0; i < 3; i++ {
		proof.LRO[i].Unmarshal(s[offset : offset+point_size])
		offset += point_size
	}

	// uint256 h_0_x;
	// uint256 h_0_y;
	// uint256 h_1_x;
	// uint256 h_1_y;
	// uint256 h_2_x;
	// uint256 h_2_y;
	for i := 0; i < 3; i++ {
		proof.H[i].Unmarshal(s[offset : offset+point_size])
		offset += point_size
	}

	// uint256 l_at_zeta;
	// uint256 r_at_zeta;
	// uint256 o_at_zeta;
	// uint256 s1_at_zeta;
	// uint256 s2_at_zeta;
	for i := 2; i < 7; i++ {
		proof.BatchedProof.ClaimedValues[i].SetBytes(s[offset : offset+fr_size])
		offset += fr_size
	}

	// uint256 grand_product_commitment_x;
	// uint256 grand_product_commitment_y;
	proof.Z.Unmarshal(s[offset : offset+point_size])
	offset += point_size

	// uint256 grand_product_at_zeta_omega;
	proof.ZShiftedOpening.ClaimedValue.SetBytes(s[offset : offset+fr_size])
	offset += fr_size

	// uint256 quotient_polynomial_at_zeta;
	// uint256 linearization_polynomial_at_zeta;
	proof.BatchedProof.ClaimedValues[0].SetBytes(s[offset : offset+fr_size])
	offset += fr_size
	proof.BatchedProof.ClaimedValues[1].SetBytes(s[offset : offset+fr_size])
	offset += fr_size

	// uint256 opening_at_zeta_proof_x;
	// uint256 opening_at_zeta_proof_y;
	proof.BatchedProof.H.Unmarshal(s[offset : offset+point_size])
	offset += point_size

	// uint256 opening_at_zeta_omega_proof_x;
	// uint256 opening_at_zeta_omega_proof_y;
	proof.ZShiftedOpening.H.Unmarshal(s[offset : offset+point_size])
	offset += point_size

	// uint256[] selector_commit_api_at_zeta;
	// uint256[] wire_committed_commitments;
	for i := 0; i < nbCommits; i++ {
		proof.BatchedProof.ClaimedValues[7+i].SetBytes(s[offset : offset+fr_size])
		offset += fr_size
	}

	for i := 0; i < nbCommits; i++ {
		proof.Bsb22Commitments[i].Unmarshal(s[offset : offset+point_size])
		offset += point_size
	}

	return proof
}
