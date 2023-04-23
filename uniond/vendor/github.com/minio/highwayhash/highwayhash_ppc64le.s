//
// Minio Cloud Storage, (C) 2018 Minio, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//+build !noasm,!appengine

#include "textflag.h"

// Definition of registers
#define V0_LO    VS32
#define V0_LO_   V0
#define V0_HI    VS33
#define V0_HI_   V1
#define V1_LO    VS34
#define V1_LO_   V2
#define V1_HI    VS35
#define V1_HI_   V3
#define MUL0_LO  VS36
#define MUL0_LO_ V4
#define MUL0_HI  VS37
#define MUL0_HI_ V5
#define MUL1_LO  VS38
#define MUL1_LO_ V6
#define MUL1_HI  VS39
#define MUL1_HI_ V7

// Message
#define MSG_LO   VS40
#define MSG_LO_  V8
#define MSG_HI   VS41

// Constants
#define ROTATE   VS42
#define ROTATE_  V10
#define MASK     VS43
#define MASK_    V11

// Temps
#define TEMP1    VS44
#define TEMP1_   V12
#define TEMP2    VS45
#define TEMP2_   V13
#define TEMP3    VS46
#define TEMP3_   V14
#define TEMP4_   V15
#define TEMP5_   V16
#define TEMP6_   V17
#define TEMP7_   V18

// Regular registers
#define STATE     R3
#define MSG_BASE  R4
#define MSG_LEN   R5
#define CONSTANTS R6
#define P1        R7
#define P2        R8
#define P3        R9
#define P4        R10
#define P5        R11
#define P6        R12
#define P7        R14 // avoid using R13

TEXT ·updatePpc64Le(SB), NOFRAME|NOSPLIT, $0-32
	MOVD state+0(FP), STATE
	MOVD msg_base+8(FP), MSG_BASE
	MOVD msg_len+16(FP), MSG_LEN  // length of message

	// Sanity check for length
	CMPU MSG_LEN, $31
	BLE  complete

	// Setup offsets
	MOVD $16, P1
	MOVD $32, P2
	MOVD $48, P3
	MOVD $64, P4
	MOVD $80, P5
	MOVD $96, P6
	MOVD $112, P7

	// Load state
	LXVD2X   (STATE)(R0), V0_LO
	LXVD2X   (STATE)(P1), V0_HI
	LXVD2X   (STATE)(P2), V1_LO
	LXVD2X   (STATE)(P3), V1_HI
	LXVD2X   (STATE)(P4), MUL0_LO
	LXVD2X   (STATE)(P5), MUL0_HI
	LXVD2X   (STATE)(P6), MUL1_LO
	LXVD2X   (STATE)(P7), MUL1_HI
	XXPERMDI V0_LO, V0_LO, $2, V0_LO
	XXPERMDI V0_HI, V0_HI, $2, V0_HI
	XXPERMDI V1_LO, V1_LO, $2, V1_LO
	XXPERMDI V1_HI, V1_HI, $2, V1_HI
	XXPERMDI MUL0_LO, MUL0_LO, $2, MUL0_LO
	XXPERMDI MUL0_HI, MUL0_HI, $2, MUL0_HI
	XXPERMDI MUL1_LO, MUL1_LO, $2, MUL1_LO
	XXPERMDI MUL1_HI, MUL1_HI, $2, MUL1_HI

	// Load asmConstants table pointer
	MOVD    $·asmConstants(SB), CONSTANTS
	LXVD2X  (CONSTANTS)(R0), ROTATE
	LXVD2X  (CONSTANTS)(P1), MASK
	XXLNAND MASK, MASK, MASK

loop:
	// Main highwayhash update loop
	LXVD2X   (MSG_BASE)(R0), MSG_LO
	VADDUDM  V0_LO_, MUL1_LO_, TEMP1_
	VRLD     V0_LO_, ROTATE_, TEMP2_
	VADDUDM  MUL1_HI_, V0_HI_, TEMP3_
	LXVD2X   (MSG_BASE)(P1), MSG_HI
	ADD      $32, MSG_BASE, MSG_BASE
	XXPERMDI MSG_LO, MSG_LO, $2, MSG_LO
	XXPERMDI MSG_HI, MSG_HI, $2, V0_LO
	VADDUDM  MSG_LO_, MUL0_LO_, MSG_LO_
	VADDUDM  V0_LO_, MUL0_HI_, V0_LO_
	VADDUDM  MSG_LO_, V1_LO_, V1_LO_
	VSRD     V0_HI_, ROTATE_, MSG_LO_
	VADDUDM  V0_LO_, V1_HI_, V1_HI_
	VPERM    V1_LO_, V1_LO_, MASK_, V0_LO_
	VMULOUW  V1_LO_, TEMP2_, TEMP2_
	VPERM    V1_HI_, V1_HI_, MASK_, TEMP7_
	VADDUDM  V0_LO_, TEMP1_, V0_LO_
	VMULOUW  V1_HI_, MSG_LO_, MSG_LO_
	VADDUDM  TEMP7_, TEMP3_, V0_HI_
	VPERM    V0_LO_, V0_LO_, MASK_, TEMP6_
	VRLD     V1_LO_, ROTATE_, TEMP4_
	VSRD     V1_HI_, ROTATE_, TEMP5_
	VPERM    V0_HI_, V0_HI_, MASK_, TEMP7_
	XXLXOR   MUL0_LO, TEMP2, MUL0_LO
	VMULOUW  TEMP1_, TEMP4_, TEMP1_
	VMULOUW  TEMP3_, TEMP5_, TEMP3_
	XXLXOR   MUL0_HI, MSG_LO, MUL0_HI
	XXLXOR   MUL1_LO, TEMP1, MUL1_LO
	XXLXOR   MUL1_HI, TEMP3, MUL1_HI
	VADDUDM  TEMP6_, V1_LO_, V1_LO_
	VADDUDM  TEMP7_, V1_HI_, V1_HI_

	SUB  $32, MSG_LEN, MSG_LEN
	CMPU MSG_LEN, $32
	BGE  loop

	// Save state
	XXPERMDI V0_LO, V0_LO, $2, V0_LO
	XXPERMDI V0_HI, V0_HI, $2, V0_HI
	XXPERMDI V1_LO, V1_LO, $2, V1_LO
	XXPERMDI V1_HI, V1_HI, $2, V1_HI
	XXPERMDI MUL0_LO, MUL0_LO, $2, MUL0_LO
	XXPERMDI MUL0_HI, MUL0_HI, $2, MUL0_HI
	XXPERMDI MUL1_LO, MUL1_LO, $2, MUL1_LO
	XXPERMDI MUL1_HI, MUL1_HI, $2, MUL1_HI
	STXVD2X  V0_LO, (STATE)(R0)
	STXVD2X  V0_HI, (STATE)(P1)
	STXVD2X  V1_LO, (STATE)(P2)
	STXVD2X  V1_HI, (STATE)(P3)
	STXVD2X  MUL0_LO, (STATE)(P4)
	STXVD2X  MUL0_HI, (STATE)(P5)
	STXVD2X  MUL1_LO, (STATE)(P6)
	STXVD2X  MUL1_HI, (STATE)(P7)

complete:
	RET

// Constants table
DATA ·asmConstants+0x0(SB)/8, $0x0000000000000020
DATA ·asmConstants+0x8(SB)/8, $0x0000000000000020
DATA ·asmConstants+0x10(SB)/8, $0x070806090d0a040b // zipper merge constant
DATA ·asmConstants+0x18(SB)/8, $0x000f010e05020c03 // zipper merge constant

GLOBL ·asmConstants(SB), 8, $32
