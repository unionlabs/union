// Copyright (c) 2015 Klaus Post, released under MIT License. See LICENSE file.

//+build arm64,!gccgo,!noasm,!appengine

// See https://www.kernel.org/doc/Documentation/arm64/cpu-feature-registers.txt

// func getMidr
TEXT ·getMidr(SB), 7, $0
	WORD $0xd5380000    // mrs x0, midr_el1         /* Main ID Register */
	MOVD R0, midr+0(FP)
	RET

// func getProcFeatures
TEXT ·getProcFeatures(SB), 7, $0
	WORD $0xd5380400            // mrs x0, id_aa64pfr0_el1  /* Processor Feature Register 0 */
	MOVD R0, procFeatures+0(FP)
	RET

// func getInstAttributes
TEXT ·getInstAttributes(SB), 7, $0
	WORD $0xd5380600            // mrs x0, id_aa64isar0_el1 /* Instruction Set Attribute Register 0 */
	WORD $0xd5380621            // mrs x1, id_aa64isar1_el1 /* Instruction Set Attribute Register 1 */
	MOVD R0, instAttrReg0+0(FP)
	MOVD R1, instAttrReg1+8(FP)
	RET

