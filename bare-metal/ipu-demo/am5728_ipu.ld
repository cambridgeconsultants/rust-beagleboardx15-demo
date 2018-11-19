/* # Developer notes

- Symbols that start with a double underscore (__) are considered "private"

- Symbols that start with a single underscore (_) are considered "semi-public"; they can be
  overridden in a user linker script, but should not be referred from user code (e.g. `extern "C" {
  static mut __sbss }`).

- `EXTERN` forces the linker to keep a symbol in the final binary. We use this to make sure a
  symbol if not dropped if it appears in or near the front of the linker arguments and "it's not
  needed" by any of the preceding objects (linker arguments)

- `PROVIDE` is used to provide default values that can be overridden by a user linker script

- On alignment: it's important for correctness that the VMA boundaries of both .bss and .data *and*
  the LMA of .data are all 4-byte aligned. These alignments are assumed by the RAM initialization
  routine. There's also a second benefit: 4-byte aligned boundaries means that you won't see
  "Address (..) is out of bounds" in the disassembly produced by `objdump`.
*/

MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* For the AM5728 IPU1 */
  /* We don't really have FLASH and RAM, just DDR
     but we keep the two segments to compatibility with cortex-m-rt */
  FLASH    (RWX): ORIGIN = 0x00000000, LENGTH = 1M
  RAM      (RW) : ORIGIN = 0x80000000, LENGTH = 5M
  IPC_DATA (RW) : ORIGIN = 0x9F000000, LENGTH = 1M,
  L2RAM    (RWX): ORIGIN = 0x20000000, LENGTH = 64K
}

/* # Entry point = reset vector */
ENTRY(ResetAM5728);
EXTERN(__RESET_VECTOR_AM5728); /* depends on the `ResetAM5728` symbol */

/* # Exception vectors */
/* This is effectively weak aliasing at the linker level */
/* The user can override any of these aliases by defining the corresponding symbol themselves (cf.
   the `exception!` macro) */
EXTERN(__EXCEPTIONS); /* depends on all the these PROVIDED symbols */

EXTERN(DefaultHandler);

PROVIDE(NonMaskableInt = DefaultHandler);
EXTERN(HardFault);
PROVIDE(MemoryManagement = DefaultHandler);
PROVIDE(BusFault = DefaultHandler);
PROVIDE(UsageFault = DefaultHandler);
PROVIDE(SecureFault = DefaultHandler);
PROVIDE(SVCall = DefaultHandler);
PROVIDE(DebugMonitor = DefaultHandler);
PROVIDE(PendSV = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);

/* # Interrupt vectors */
EXTERN(__INTERRUPTS); /* `static` variable similar to `__EXCEPTIONS` */

/* # Pre-initialization function */
/* If the user overrides this using the `pre_init!` macro or by creating a `__pre_init` function,
   then the function this points to will be called before the RAM is initialized. */
PROVIDE(__pre_init = DefaultPreInit);

/* # User overridable symbols I */
/* Lets the user place the stack in a different RAM region */
PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));

/* # Sections */
SECTIONS
{
  /* ## Sections in FLASH */
  /* ### Vector table */
  .vector_table ORIGIN(FLASH) : ALIGN(4)
  {
    /* Initial Stack Pointer (SP) value */
    __STACK_START = .; /* Just to get a nicer name in the disassembly */
    LONG(_stack_start);

    /* Reset vector */
    KEEP(*(.vector_table.reset_vector_am5728)); /* this is `__RESET_VECTOR_AM5728` symbol */
    __reset_vector = ABSOLUTE(.);

    /* Exceptions */
    KEEP(*(.vector_table.exceptions)); /* this is `__EXCEPTIONS` symbol */
    __eexceptions = ABSOLUTE(.);

    /* Device specific interrupts */
    KEEP(*(.vector_table.interrupts)); /* this is `__INTERRUPTS` symbol */
    __einterrupts = ABSOLUTE(.);
  } > FLASH

  /* ### .text */
  .text _stext :
  {
    *(.text .text.*);
    __etext = ABSOLUTE(.);
  } > FLASH

  /* ### .rodata */
  .rodata :
  {
    . = ALIGN(4); /* 4-byte align the start (VMA) of this section */
    /* __srodata = ABSOLUTE(.); */

    KEEP(*(.vector_table.reset_vector)); /* this is `__RESET_VECTOR` symbol */
    *(.rodata .rodata.*);

    . = ALIGN(4); /* 4-byte align the end (VMA) of this section */
    __erodata = ABSOLUTE(.);
  } > FLASH

  /* ## Sections in RAM */
  /* ### .data */
  .data :
  {
    . = ALIGN(4); /* 4-byte align the start (VMA) of this section */
    __sdata = ABSOLUTE(.);
    __edata = ABSOLUTE(.);

    *(.data .data.*);

    . = ALIGN(4); /* 4-byte align the end (VMA) of this section */
  } > RAM

  /* ### .bss */
  .bss :
  {
    . = ALIGN(4); /* 4-byte align the start (VMA) of this section */
    __sbss = ABSOLUTE(.);

    *(.bss .bss.*);

    . = ALIGN(4); /* 4-byte align the end (VMA) of this section */
    __ebss = ABSOLUTE(.);
  } > RAM

  /* ## Fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect relocatable code in
     the input files and raise an error if relocatable code is found */
  .got :
  {
    __sgot = ABSOLUTE(.);
    KEEP(*(.got .got.*));
    __egot = ABSOLUTE(.);
  } > FLASH

  /* This is how we communicate with the kernel */
  .ipc_data : {
      KEEP(*(.tracebuffer .tracebuffer.*))
      KEEP(*(.ipc_data .ipc_data.*))
  } > IPC_DATA

  /* The kernel looks for a section with this name */
  .resource_table : {
      KEEP(*(.resource_table))
  } > FLASH

  /* The kernel looks for a section with this name */
  .version : {
      KEEP(*(.version))
  } > FLASH

  /* ## Discarded sections */
  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx.*);
  }
}

/* # User overridable symbols II */
/* (The user overridable symbols are split in two parts because LLD demands that the RHS of PROVIDE
    to be defined before the PROVIDE invocation) */
/* Lets the user override this to place .text a bit further than the vector table. Required by
microcontrollers that store their configuration right after the vector table. */
PROVIDE(_stext = __einterrupts);

/* Default IRQ handlers as weak symbols */
PROVIDE(Ipu1Irq16 = DefaultHandler);
PROVIDE(Ipu1Irq17 = DefaultHandler);
PROVIDE(Ipu1Irq18 = DefaultHandler);
PROVIDE(Ipu1Irq19 = DefaultHandler);
PROVIDE(Ipu1Irq20 = DefaultHandler);
PROVIDE(Ipu1Irq21 = DefaultHandler);
PROVIDE(Ipu1Irq22 = DefaultHandler);
PROVIDE(Ipu1Irq23 = DefaultHandler);
PROVIDE(Ipu1Irq24 = DefaultHandler);
PROVIDE(Ipu1Irq25 = DefaultHandler);
PROVIDE(Ipu1Irq26 = DefaultHandler);
PROVIDE(Ipu1Irq27 = DefaultHandler);
PROVIDE(Ipu1Irq28 = DefaultHandler);
PROVIDE(Ipu1Irq29 = DefaultHandler);
PROVIDE(Ipu1Irq30 = DefaultHandler);
PROVIDE(Ipu1Irq31 = DefaultHandler);
PROVIDE(Ipu1Irq32 = DefaultHandler);
PROVIDE(Ipu1Irq33 = DefaultHandler);
PROVIDE(Ipu1Irq34 = DefaultHandler);
PROVIDE(Ipu1Irq35 = DefaultHandler);
PROVIDE(Ipu1Irq36 = DefaultHandler);
PROVIDE(Ipu1Irq37 = DefaultHandler);
PROVIDE(Ipu1Irq38 = DefaultHandler);
PROVIDE(Ipu1Irq39 = DefaultHandler);
PROVIDE(Ipu1Irq40 = DefaultHandler);
PROVIDE(Ipu1Irq41 = DefaultHandler);
PROVIDE(Ipu1Irq42 = DefaultHandler);
PROVIDE(Ipu1Irq43 = DefaultHandler);
PROVIDE(Ipu1Irq44 = DefaultHandler);
PROVIDE(Ipu1Irq45 = DefaultHandler);
PROVIDE(Ipu1Irq46 = DefaultHandler);
PROVIDE(Ipu1Irq47 = DefaultHandler);
PROVIDE(Ipu1Irq48 = DefaultHandler);
PROVIDE(Ipu1Irq49 = DefaultHandler);
PROVIDE(Ipu1Irq50 = DefaultHandler);
PROVIDE(Ipu1Irq51 = DefaultHandler);
PROVIDE(Ipu1Irq52 = DefaultHandler);
PROVIDE(Ipu1Irq53 = DefaultHandler);
PROVIDE(Ipu1Irq54 = DefaultHandler);
PROVIDE(Ipu1Irq55 = DefaultHandler);
PROVIDE(Ipu1Irq56 = DefaultHandler);
PROVIDE(Ipu1Irq57 = DefaultHandler);
PROVIDE(Ipu1Irq58 = DefaultHandler);
PROVIDE(Ipu1Irq59 = DefaultHandler);
PROVIDE(Ipu1Irq60 = DefaultHandler);
PROVIDE(Ipu1Irq61 = DefaultHandler);
PROVIDE(Ipu1Irq62 = DefaultHandler);
PROVIDE(Ipu1Irq63 = DefaultHandler);
PROVIDE(Ipu1Irq64 = DefaultHandler);
PROVIDE(Ipu1Irq65 = DefaultHandler);
PROVIDE(Ipu1Irq66 = DefaultHandler);
PROVIDE(Ipu1Irq67 = DefaultHandler);
PROVIDE(Ipu1Irq68 = DefaultHandler);
PROVIDE(Ipu1Irq69 = DefaultHandler);
PROVIDE(Ipu1Irq70 = DefaultHandler);
PROVIDE(Ipu1Irq71 = DefaultHandler);
PROVIDE(Ipu1Irq72 = DefaultHandler);
PROVIDE(Ipu1Irq73 = DefaultHandler);
PROVIDE(Ipu1Irq74 = DefaultHandler);
PROVIDE(Ipu1Irq75 = DefaultHandler);
PROVIDE(Ipu1Irq76 = DefaultHandler);
PROVIDE(Ipu1Irq77 = DefaultHandler);
PROVIDE(Ipu1Irq78 = DefaultHandler);
PROVIDE(Ipu1Irq79 = DefaultHandler);

/* # Hardcoded symbols */
/* Place `.bss` at the start of the RAM region */
__sidata = LOADADDR(.data);
/* Place the heap right after `.bss` and `.data` */
__sheap = __ebss;

/* # Sanity checks */

/* Do not exceed this mark in the error messages below                                    | */
ASSERT(__reset_vector == ORIGIN(FLASH) + 0x8, "
cortex-m-rt: The reset vector is missing. This is a bug in cortex-m-rt. Please file a bug
report at: https://github.com/japaric/cortex-m-rt/issues");

ASSERT(__eexceptions - ORIGIN(FLASH) == 0x40, "
cortex-m-rt: The exception vectors are missing. This is a bug in cortex-m-rt. Please file
a bug report at: https://github.com/japaric/cortex-m-rt/issues");

ASSERT(__sheap >= __ebss, "
cortex-m-rt: The heap overlaps with the .bss section. This is a bug in cortex-m-rt. Please
file a bug report at: https://github.com/japaric/cortex-m-rt/issues");

ASSERT(__sheap >= __edata, "
cortex-m-rt: The heap overlaps with the .data section. This is a bug in cortex-m-rt.
Please file a bug report at: https://github.com/japaric/cortex-m-rt/issues");

ASSERT(__einterrupts - __eexceptions > 0, "
cortex-m-rt: The interrupt vectors are missing. Possible solutions, from most likely to
less likely:
- Link to a device crate
- Disable the 'device' feature of cortex-m-rt to build a generic application (a dependency
  may be enabling it)
- Supply the interrupt handlers yourself. Check the documentation for details.");

ASSERT(__einterrupts <= _stext, "
cortex-m-rt: The '.text' section can't be placed inside the '.vector_table' section. Set
'_stext' to an address greater than '__einterrupts' (cf. `nm` output)");

ASSERT(_stext < ORIGIN(FLASH) + LENGTH(FLASH), "
cortex-m-rt The '.text' section must be placed inside the FLASH memory. Set '_stext' to an
address smaller than 'ORIGIN(FLASH) + LENGTH(FLASH)");

/* This has been temporarily omitted because it's not supported by LLD */
/* ASSERT(__sbss % 4 == 0 && __ebss % 4 == 0, " */
/* .bss is not 4-byte aligned at its boundaries. This is a cortex-m-rt bug."); */

/* ASSERT(__sdata % 4 == 0 && __edata % 4 == 0, " */
/* .data is not 4-byte aligned at its boundaries. This is a cortex-m-rt bug."); */

/* ASSERT(__sidata % 4 == 0, " */
/* __sidata is not 4-byte aligned. This is a cortex-m-rt bug."); */

ASSERT(__sgot == __egot, "
.got section detected in the input object files. Dynamic relocations are not supported.
If you are linking to C code compiled using the `cc` crate then modify your build script
to compile the C code _without_ the -fPIC flag. See the documentation of the
`cc::Build.pic` method for details.");
/* Do not exceed this mark in the error messages above                                    | */

ASSERT(__einterrupts - __eexceptions <= 0x3c0, "
There can't be more than 240 interrupt handlers. This may be a bug in
your device crate, or you may have registered more than 240 interrupt
handlers.");

