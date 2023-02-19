SECTIONS {

    . = 0x080131e8;
    .data : { *(.data.wavesA) }
    . = 0x0801546c;
    .data : { *(.data.wavesB) }
    . = 0x080174ec;
    .data : { *(.data.wavesC) }
    . = 0x0801915c;
    .data : { *(.data.wavesD) }
    . = 0x0801abc4;
    .data : { *(.data.wavesE) }
    . = 0x0801ca3c;
    .data : { *(.data.wavesF) }

	. = 0x0801ed80;
	.text : { *(.text._osc_rand) }
	. = 0x0801edb8;
	.text : { *(.text._osc_white) }
}
