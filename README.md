# Texas Instruments AM5728 IPU Firmware

The TI AM5728 contains a number of processing cores:

* Two ARM Cortex A15s
* Two ARM Cortex M4s clusters: IPU1 and IPU2
	- Each cluster has two cores, C0 and C1
* Two C66x DSP cores

This SoC can be found on the AM572x EVM, and the Beagleboard X15.

Linux boots on the A15s from U-Boot, as you would expect. Running code on the other CPUs uses a Linux feature called "remote processors" (also called "remote proc" or just "rproc").

You can boot a vanilla kernel built with buildroot, but a number of features are missing including, it seems, omap-rpoc support. More research TBD.

The Beagleboard X15 ships with 4.9.35-ti-r44, which lives at http://git.ti.com/gitweb/?p=ti-linux-kernel/ti-linux-kernel.git;a=summary. To save pulling the entire tree (it's huge) I took a release tarball from a [Github mirror](https://github.com/RobertCNelson/ti-linux-kernel/releases).

## Compiling


### Building the IPU application

This application is built in Rust. Currently this demo requires the night release of the Rust compiler as it uses in-line assembler to sleep the second core of IPU1, without using the stack.

```
$ cargo +nightly build --release
...
$ scp ./target/thumbv7em-none-eabi/release/ipu-demo-rs root@BeagleBoard-X15:/lib/firmware/dra7-ipu1-fw.xem4
```

### Building the socket demo

This application is built in Rust, and compiles using the stable compiler. You can use either the ARMv7 Linux musl-c or the ARMv7 Linux glibc targets - the default is `armv7-unknown-linux-musleabihf`.

```
$ cargo build --release
...
$ scp ./target/armv7-unknown-linux-musleabihf/release/socket-demo debian@BeagleBoard-X15:
```

## Running on-target

Enable some debug (optional):

```
root@BeagleBoard-X15:~# echo -n "file net/rpmsg/* +p" > /sys/kernel/debug/dynamic_debug/control
root@BeagleBoard-X15:~# echo -n "file drivers/rpmsg/* +p" > /sys/kernel/debug/dynamic_debug/control
```

Reload the IPU:

```
root@BeagleBoard-X15:~# echo 58820000.ipu > /sys/bus/platform/drivers/omap-rproc/unbind
root@BeagleBoard-X15:~# sleep 1
root@BeagleBoard-X15:~# echo 58820000.ipu > /sys/bus/platform/drivers/omap-rproc/bind
```

If the IPU crashes, don't auto restart (so we can debug it):

```
root@BeagleBoard-X15:~# echo "disabled" > /sys/kernel/debug/remoteproc/remoteproc0/recovery
```

This command will show you the last 30 lines of trace messages, every two seconds:

```
root@BeagleBoard-X15:~# watch tail -n 30 /sys/kernel/debug/remoteproc/remoteproc0/trace0
```

You can now run the socket demo as a regular user. This will exchange a number of messages with the IPU:

```
debian@BeagleBoard-X15:~$ ./socket-demo
Running...

Waiting for socket...
999: Sent: 1000, Recv: 1000
1999: Sent: 2000, Recv: 2000
2999: Sent: 3000, Recv: 3000
3999: Sent: 4000, Recv: 4000
4999: Sent: 5000, Recv: 5000
5999: Sent: 6000, Recv: 6000
6999: Sent: 7000, Recv: 7000
7999: Sent: 8000, Recv: 8000
8999: Sent: 9000, Recv: 9000
9999: Sent: 10000, Recv: 10000
Final Total: Sent: 10000, Recv: 10000
Test complete.
debian@BeagleBoard-X15:~$
```

## Technical Details

### Loading the IPU from the MPU

Booting the IPU from Linux is fairly straightforward. The kernel looks for specially named files in `/lib/firmware` on startup and if they are found, it loads them into RAM and boots the relevant processor.

The files are called:

* dra7-dsp1-fw.xe66
* dra7-dsp2-fw.xe66
* dra7-ipu1-fw.xem4
* dra7-ipu2-fw.xem4

You can also manually stop and reload a CPU with the following commands:

```bash
cd /sys/bus/platform/drivers/omap-rproc/
echo 55020000.ipu > unbind
cp /home/root/new-binary.elf /lib/firmware/dra7-ipu2-fw.xem4
echo 55020000.ipu > bind
```

The magic strings you pass to `bind` and `unbind` are defined in `drivers/remoteproc/omap_remoteproc.c`:

```c
static const struct omap_rproc_dev_data dra7_rproc_dev_data[] = {
	{
		.device_name	= "40800000.dsp",
		.fw_name	= "dra7-dsp1-fw.xe66",
	},
	{
		.device_name	= "41000000.dsp",
		.fw_name	= "dra7-dsp2-fw.xe66",
	},
	{
		.device_name	= "55020000.ipu",
		.fw_name	= "dra7-ipu2-fw.xem4",
	},
	{
		.device_name	= "58820000.ipu",
		.fw_name	= "dra7-ipu1-fw.xem4",
	},
	{
		/* sentinel */
	},
};
```

It's important to note that the IPU's 32-bit address space is provided by two MMUs: the IPUx_UNICACHE_MMU and the IPUx_MMU. You can use these MMUs to map regions from other parts of the chip in the IPU's addresses space.

### Firmware file format

If you try and load any old ELF file, you will get this error:

```
[  351.979688] omap-rproc 58820000.ipu: assigned reserved memory node ipu1_cma@9d000000
[  351.981197] remoteproc remoteproc0: 58820000.ipu is available
[  351.981633] remoteproc remoteproc0: powering up 58820000.ipu
[  351.981652] remoteproc remoteproc0: Booting fw image dra7-ipu1-fw.xem4, size 5884
[  351.981781] omap-iommu 58882000.mmu: 58882000.mmu: version 2.1
[  351.981802] remoteproc remoteproc0: Failed to find resource table
```

What's a *resource table*? It's a special section in the ELF file named `.resource_table` and it describes to the Linux kernel the various memory address ranges you would like your remote processor to be able to see. The section actually contains a C structure called `struct resource_table` defined in `include\linux\remoteproc.h`.

The various sections are:

* Carveouts - a request to the host for a piece of physically contiguous memory
* Device Memory - a request to the host for a range of memory mapped IO registers
* Trace Buffers - information about a trace buffer the remote processor will write to
* VirtIO Devices - this seems to allow the host to communicate with the remote processor through various ring buffers, as a special class of VirtIO device

The carveouts the IPU firmware asks for must match the Carveout Memory Area (CMA) regions specified in the Device Tree. TI have some defaults [listed on their Wiki](http://processors.wiki.ti.com/index.php/IPC_Resource_customTable) and I don't see a reason to change them.

### The IPU boot process

The IPU boot process is shrouded in mystery. The following section is the result of four days intensive research, debugging and reverse-engineering.

The TRM for the AM5728 (SPRUHZ6J) makes a number of statements in *Section 7 - Dual Cortex-M4 IPU Subsystem* which don't appear to be correct, and also leaves out some important information. The first is that there is an IPU memory map, but it's not referred to in this section - see Table 2-9, or see a copy below:

| Region Name | Start_Address (hex) | End_Address (hex) | Size | Description |
| ----------- | ------------------- | ----------------- | ---- | ----------- |
| IPU_BOOT_SPACE(1) | 0x0000_0000 | 0x0000_3FFF | 16KiB | IPU boot space |
| L3_MAIN map | 0x0000_0000 | 0x1FFF_FFFF | 512KiB | See Table 2-1 |
| IPU_BITBAND_REGION1 | 0x2000_0000 | 0x200F_FFFF | 1MiB | IPU bit-band region 1 |
| Reserved | 0x2010_0000 | 0x21FF_FFFF | 31MiB | Reserved |
| IPU_BITBAND_ALIAS1 | 0x2200_0000 | 0x23FF_FFFF | 32MiB | IPU bit-band alias 1 |
| L3_MAIN map | 0x2400_0000 | 0x3FFF_FFFF | 448MiB | See Table 2-1 |
| IPU_BITBAND_REGION2 | 0x4000_0000 | 0x400F_FFFF | 1MiB | IPU bit-band region 2 |
| Reserved | 0x4010_0000 | 0x402F_FFFF | 2MiB | Reserved |
| L3_MAIN map | 0x4030_0000 | 0x41FF_FFFF | 30MiB | See Table 2-1 |
| IPU_BITBAND_ALIAS2 | 0x4200_0000 | 0x43FF_FFFF | 32MiB | IPU bit-band alias 2 |
| L3_MAIN map | 0x4400_0000 | 0x54FF_FFFF | 285MiB | See Table 2-1 |
| IPU_ROM(2) | 0x5500_0000 | 0x5500_3FFF | 16KiB | IPU_ROM |
| IPU_RAM(2) | 0x5502_0000 | 0x5502_FFFF | 64KiB | IPU_RAM |
| IPU_UNICACHE_MMU(2) | 0x5508_0000 | 0x5508_0FFF | 4KiB | IPU_UNICACHE_MMU config registers |
| IPU_WUGEN(2) | 0x5508_1000 | 0x5508_1FFF | 4KiB | IPU_WUGEN config registers |
| IPU_MMU(2) | 0x5508_2000 | 0x5508_2FFF | 4KiB | IPU_MMU config registers |
| Reserved | 0x5508_3000 | 0x55FF_FFFF | 16MiB | Reserved |
| L3_MAIN map | 0x5600_0000 | 0xDFFF_FFFF | 2 |,3GiB See Table 2-1 |
| Reserved | 0xE000_0000 | 0xE000_0FFF | 4KiB | Reserved |
| IPU_C0_DWT | 0xE000_1000 | 0xE000_1FFF | 4KiB | IPU_C0_DWT config registers |
| IPU_C0_FPB | 0xE000_2000 | 0xE000_2FFF | 4KiB | IPU_C0_FPB config registers |
| IPU_C0_INTC | 0xE000_E000 | 0xE000_EFFF | 4KiB | IPU_C0_INTC config registers |
| IPU_C0_ICECRUSHER | 0xE004_2000 | 0xE004_2FFF | 4KiB | IPU_C0_ICECRUSHER config registers |
| IPU_C0_RW_TABLE | 0xE00F_E000 | 0xE00F_EFFF | 4KiB | IPU_C0 RW table |
| IPU_C0_ROM_TABLE | 0xE00F_F000 | 0xE00F_FFFF | 4KiB | IPU_C0 ROM table |
| IPU_C1_DWT | 0xE000_1000 | 0xE000_1FFF | 4KiB | IPU_C1_DWT config registers |
| IPU_C1_FPB | 0xE000_2000 | 0xE000_2FFF | 4KiB | IPU_C1_FPB config registers |
| IPU_C1_INTC | 0xE000_E000 | 0xE000_EFFF | 4KiB | IPU_C1_INTC config registers |
| IPU_C1_ICECRUSHER | 0xE004_2000 | 0xE004_2FFF | 4KiB | IPU_C1_ICECRUSHER config registers |
| IPU_C1_RW_TABLE | 0xE00F_E000 | 0xE00F_EFFF | 4KiB | IPU_C1 RW table |
| IPU_C1_ROM_TABLE | 0xE00F_F000 | 0xE00F_FFFF | 4KiB | IPU_C1 ROM table |
| L3_MAIN map | 0xE010_0000 | 0xFFFF_FFFF | 511MiB | See Table 2-1 |

There is literally no other reference to the mysterious *IPU_BOOT_SPACE* region in this document, but some [TI Training Material](https://training.ti.com/sites/default/files/docs/Running_RTOS_on_Cortex_M4_SLIDES.pdf) makes the following statement:

> At reset, the MMU is loaded with Page 0, which forces the L2 RAM (0x5502_0000) to be address 0x0. Page 1 is loaded with the physical address of the shared cache MMU register and IPU_WUGEN registers to the virtual address 0x4000_0000.

Both IPU cores, being standard Cortex-M4s, will boot from a vector table located at 0x0000_0000. This is in the usual Cortex-M format:

```c
uint32_t vector_table[] = {
    // Stack Pointer
    (unsigned long) &_stack_top,
    // Reset handler
    (unsigned long) rst_handler,
    // Standard Cortex-M4 exception handlers
    (unsigned long) empty_def_handler,      // NMI handler.                     2
    (unsigned long) empty_def_handler,      // hard fault handler.              3
    (unsigned long) empty_def_handler,      // Memory Management Fault          4
    (unsigned long) empty_def_handler,      // Bus Fault                        5
    (unsigned long) empty_def_handler,      // Usage Fault                      6
    (unsigned long) empty_def_handler,      // Reserved                         7
    (unsigned long) empty_def_handler,      // Reserved                         8
    (unsigned long) empty_def_handler,      // Reserved                         9
    (unsigned long) empty_def_handler,      // Reserved                         10
    (unsigned long) empty_def_handler,      // SV call                          11
    (unsigned long) empty_def_handler,      // Debug monitor                    12
    (unsigned long) empty_def_handler,      // Reserved                         13
    (unsigned long) empty_def_handler,      // PendSV                           14
    (unsigned long) empty_def_handler,      // SysTick                          15
    // Now 64 CPU specific interrupt handlers
    (unsigned long) empty_def_handler,      // ISR 0x00
    ...
}
```

If you disassemble a TI IPU image, you will see that the vector table is in-fact located at 0x0000_0400 and in an SMP image, there is a second table at 0x0000_0800 for the second core. The linker command scripts don't seem to make any reference to this mysterious IPU_BOOT_SPACE at 0x0.

It turns out, the linker scripts don't name a region but do link a piece of code to address 0x0. That code (Core_smp_asm.sv7M) looks like this:

```asm
;
;  Copyright (c) 2012, Texas Instruments Incorporated
;  All rights reserved.
;
;  Redistribution and use in source and binary forms, with or without
;  modification, are permitted provided that the following conditions
;  are met:
;
;  *  Redistributions of source code must retain the above copyright
;     notice, this list of conditions and the following disclaimer.
;
;  *  Redistributions in binary form must reproduce the above copyright
;     notice, this list of conditions and the following disclaimer in the
;     documentation and/or other materials provided with the distribution.
;
;  *  Neither the name of Texas Instruments Incorporated nor the names of
;     its contributors may be used to endorse or promote products derived
;     from this software without specific prior written permission.
;
;  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
;  AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
;  THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
;  PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
;  CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
;  EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
;  PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
;  OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
;  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
;  OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE,
;  EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
;
;
; ======== Core_asm.asm ========
;
;

        .thumb

        .global ti_sysbios_family_arm_ducati_Core_reset

        .sect   ".ducatiBoot"

        ; reset vectors
vecbase:
        .long   0               ; sp = not used
        .long   ti_sysbios_family_arm_ducati_Core_reset

core1sp:
        .long   0               ; Core 1 sp
core1vec
        .long   0               ; Core 1 resetVec

        .thumbfunc ti_sysbios_family_arm_ducati_Core_reset
ti_sysbios_family_arm_ducati_Core_reset:
        .asmfunc
        ldr     r0, coreid      ; point to coreid reg
        ldr     r0, [r0]        ; read coreid
        cmp     r0, #0
        bne     core1
core0:
        ; Core 0 jumps to _c_int00 immediately
        ldr     lr, reset0
        ldr     lr, [lr]        ; read core 0 reset vector
        bx      lr              ; jump to core0's c_int00
core1:
        ; Core 1 waits for "a while" to let core 0 init the system
        ldr     r0, core1vec
        cmp     r0, #0
        beq     core1           ; loop until core 0 unleashes us

        mov     r2, #core1vec-vecbase
        mov     r1, #0
        str     r1, [r2]        ; clean up for next reset

        ldr     sp, core1sp
        bx      r0              ; jump to core1's c_int00

coreid: .word   0xE00FFFE0

reset0: .word   0x00000404      ; reset vector addr for core 0

        .endasmfunc

```

It seems *ducati* is the codename for the original dual Cortex-M3 multi-media acceleration subsystem on the OMAP4. I guess the name just stuck.

Did you spot the two 32-bit values stored at `vecbase`? This is actually the vector table used to boot the CPU. They've left out the 14 exception vectors and 64 interrupt vectors and replaced it with machine code - let's hope the boot code never crashes! The reset vector at 0x0000_0004 jumps to `ti_sysbios_family_arm_ducati_Core_reset`. This routine reads address 0xE00FFFE0 (the [Peripheral ID0 register](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0439b/BCGFHGCC.html)) to determine which core is running (as both cores are executing the same code at this point). If it's zero, we assume we are Core 0, and jump to the vector at 0x0000_0404. If it's non-zero we assume we are Core 1 and wait for Core 0 to write to 0x0000_000C. When that happens, Core 1 takes a stack pointer from 0x0000_0008 and jumps to the vector written to 0x0000_000C. Once running one assumes the CPUs change their *Vector Table Offset Register* (VTOR) to point to their own vector tables, to handle any exceptions or interrupts appropriately.

### Building your own bare-metal image

Your code must complete the following steps:

* Include a `.resource_table` section.
* Include a vector table at 0x0000_0000.
* Using assembly language (to guarantee the stack pointer isn't used), determine which core is running, then jump to core specific initialisation code.
* In the core specific code, move the vector table to a chip-unique location (or just keep the second CPU shut-down).

### Modifications to linker script

We took the cortex-m-rt standard `link.x.in` linker script and added the following sections to support the AM5728 IPU:

```
$ diff link.x.in ../rust-beagleboardx15-demo/bare-metal/ipu-demo/am5728_ipu.ld
21,23c21,31
< /* Provides information about the memory layout of the device */
< /* This will be provided by the user (see `memory.x`) or by a Board Support Crate */
< INCLUDE memory.x
---
> MEMORY
> {
>   /* NOTE K = KiBi = 1024 bytes */
>   /* For the AM5728 IPU1 */
>   /* We don't really have FLASH and RAM, just DDR
>      but we keep the two segments to compatibility with cortex-m-rt */
>   FLASH    (RWX): ORIGIN = 0x00000000, LENGTH = 1M
>   RAM      (RW) : ORIGIN = 0x80000000, LENGTH = 5M
>   IPC_DATA (RW) : ORIGIN = 0x9F000000, LENGTH = 1M,
>   L2RAM    (RWX): ORIGIN = 0x20000000, LENGTH = 64K
> }
26,27c34,35
< ENTRY(Reset);
< EXTERN(__RESET_VECTOR); /* depends on the `Reset` symbol */
---
> ENTRY(ResetAM5728);
> EXTERN(__RESET_VECTOR_AM5728); /* depends on the `ResetAM5728` symbol */
50a59,63
> /* # Pre-initialization function */
> /* If the user overrides this using the `pre_init!` macro or by creating a `__pre_init` function,
>    then the function this points to will be called before the RAM is initialized. */
> PROVIDE(__pre_init = DefaultPreInit);
>
67c80
<     KEEP(*(.vector_table.reset_vector)); /* this is `__RESET_VECTOR` symbol */
---
>     KEEP(*(.vector_table.reset_vector_am5728)); /* this is `__RESET_VECTOR_AM5728` symbol */
91a105
>     KEEP(*(.vector_table.reset_vector)); /* this is `__RESET_VECTOR` symbol */
100c114
<   .data : AT(__erodata) /* LMA */
---
>   .data :
103a118
>     __edata = ABSOLUTE(.);
108d122
<     __edata = ABSOLUTE(.);
132a147,162
>   /* This is how we communicate with the kernel */
>   .ipc_data : {
>       KEEP(*(.tracebuffer .tracebuffer.*))
>       KEEP(*(.ipc_data .ipc_data.*))
>   } > IPC_DATA
>
>   /* The kernel looks for a section with this name */
>   .resource_table : {
>       KEEP(*(.resource_table))
>   } > FLASH
>
>   /* The kernel looks for a section with this name */
>   .version : {
>       KEEP(*(.version))
>   } > FLASH
>
147a178,243
> /* Default IRQ handlers as weak symbols */
> PROVIDE(Ipu1Irq16 = DefaultHandler);
> PROVIDE(Ipu1Irq17 = DefaultHandler);
> PROVIDE(Ipu1Irq18 = DefaultHandler);
> PROVIDE(Ipu1Irq19 = DefaultHandler);
> PROVIDE(Ipu1Irq20 = DefaultHandler);
> PROVIDE(Ipu1Irq21 = DefaultHandler);
> PROVIDE(Ipu1Irq22 = DefaultHandler);
> PROVIDE(Ipu1Irq23 = DefaultHandler);
> PROVIDE(Ipu1Irq24 = DefaultHandler);
> PROVIDE(Ipu1Irq25 = DefaultHandler);
> PROVIDE(Ipu1Irq26 = DefaultHandler);
> PROVIDE(Ipu1Irq27 = DefaultHandler);
> PROVIDE(Ipu1Irq28 = DefaultHandler);
> PROVIDE(Ipu1Irq29 = DefaultHandler);
> PROVIDE(Ipu1Irq30 = DefaultHandler);
> PROVIDE(Ipu1Irq31 = DefaultHandler);
> PROVIDE(Ipu1Irq32 = DefaultHandler);
> PROVIDE(Ipu1Irq33 = DefaultHandler);
> PROVIDE(Ipu1Irq34 = DefaultHandler);
> PROVIDE(Ipu1Irq35 = DefaultHandler);
> PROVIDE(Ipu1Irq36 = DefaultHandler);
> PROVIDE(Ipu1Irq37 = DefaultHandler);
> PROVIDE(Ipu1Irq38 = DefaultHandler);
> PROVIDE(Ipu1Irq39 = DefaultHandler);
> PROVIDE(Ipu1Irq40 = DefaultHandler);
> PROVIDE(Ipu1Irq41 = DefaultHandler);
> PROVIDE(Ipu1Irq42 = DefaultHandler);
> PROVIDE(Ipu1Irq43 = DefaultHandler);
> PROVIDE(Ipu1Irq44 = DefaultHandler);
> PROVIDE(Ipu1Irq45 = DefaultHandler);
> PROVIDE(Ipu1Irq46 = DefaultHandler);
> PROVIDE(Ipu1Irq47 = DefaultHandler);
> PROVIDE(Ipu1Irq48 = DefaultHandler);
> PROVIDE(Ipu1Irq49 = DefaultHandler);
> PROVIDE(Ipu1Irq50 = DefaultHandler);
> PROVIDE(Ipu1Irq51 = DefaultHandler);
> PROVIDE(Ipu1Irq52 = DefaultHandler);
> PROVIDE(Ipu1Irq53 = DefaultHandler);
> PROVIDE(Ipu1Irq54 = DefaultHandler);
> PROVIDE(Ipu1Irq55 = DefaultHandler);
> PROVIDE(Ipu1Irq56 = DefaultHandler);
> PROVIDE(Ipu1Irq57 = DefaultHandler);
> PROVIDE(Ipu1Irq58 = DefaultHandler);
> PROVIDE(Ipu1Irq59 = DefaultHandler);
> PROVIDE(Ipu1Irq60 = DefaultHandler);
> PROVIDE(Ipu1Irq61 = DefaultHandler);
> PROVIDE(Ipu1Irq62 = DefaultHandler);
> PROVIDE(Ipu1Irq63 = DefaultHandler);
> PROVIDE(Ipu1Irq64 = DefaultHandler);
> PROVIDE(Ipu1Irq65 = DefaultHandler);
> PROVIDE(Ipu1Irq66 = DefaultHandler);
> PROVIDE(Ipu1Irq67 = DefaultHandler);
> PROVIDE(Ipu1Irq68 = DefaultHandler);
> PROVIDE(Ipu1Irq69 = DefaultHandler);
> PROVIDE(Ipu1Irq70 = DefaultHandler);
> PROVIDE(Ipu1Irq71 = DefaultHandler);
> PROVIDE(Ipu1Irq72 = DefaultHandler);
> PROVIDE(Ipu1Irq73 = DefaultHandler);
> PROVIDE(Ipu1Irq74 = DefaultHandler);
> PROVIDE(Ipu1Irq75 = DefaultHandler);
> PROVIDE(Ipu1Irq76 = DefaultHandler);
> PROVIDE(Ipu1Irq77 = DefaultHandler);
> PROVIDE(Ipu1Irq78 = DefaultHandler);
> PROVIDE(Ipu1Irq79 = DefaultHandler);
>
204a301,306
>
> ASSERT(__einterrupts - __eexceptions <= 0x3c0, "
> There can't be more than 240 interrupt handlers. This may be a bug in
> your device crate, or you may have registered more than 240 interrupt
> handlers.");
>
```

## License

The original material herein is Copyright (c) 2018, Cambridge Consultants Ltd.

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
