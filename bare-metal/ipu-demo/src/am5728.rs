//! # AM5728 Drivers
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! Code specific to the AM5728 as fitted to the Beagleboard X15
//!
//! The MMU code here handles the L1 (or 'Unicache') MMU, also known as the
//! 'AMMU'. The main IPU MMU is programmed by Linux running on the Cortex-A15,
//! where it is considered to be an 'IOMMU'.

// We leave in definitions of registers we're not using for completeness.
#![allow(dead_code)]

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use super::resource_table as rt;
use cortex_m::{self, interrupt::Nr};
use volatile_register::{RO, RW};

extern "C" {
    fn Ipu1Irq16();
    fn Ipu1Irq17();
    fn Ipu1Irq18();
    fn Ipu1Irq19();
    fn Ipu1Irq20();
    fn Ipu1Irq21();
    fn Ipu1Irq22();
    fn Ipu1Irq23();
    fn Ipu1Irq24();
    fn Ipu1Irq25();
    fn Ipu1Irq26();
    fn Ipu1Irq27();
    fn Ipu1Irq28();
    fn Ipu1Irq29();
    fn Ipu1Irq30();
    fn Ipu1Irq31();
    fn Ipu1Irq32();
    fn Ipu1Irq33();
    fn Ipu1Irq34();
    fn Ipu1Irq35();
    fn Ipu1Irq36();
    fn Ipu1Irq37();
    fn Ipu1Irq38();
    fn Ipu1Irq39();
    fn Ipu1Irq40();
    fn Ipu1Irq41();
    fn Ipu1Irq42();
    fn Ipu1Irq43();
    fn Ipu1Irq44();
    fn Ipu1Irq45();
    fn Ipu1Irq46();
    fn Ipu1Irq47();
    fn Ipu1Irq48();
    fn Ipu1Irq49();
    fn Ipu1Irq50();
    fn Ipu1Irq51();
    fn Ipu1Irq52();
    fn Ipu1Irq53();
    fn Ipu1Irq54();
    fn Ipu1Irq55();
    fn Ipu1Irq56();
    fn Ipu1Irq57();
    fn Ipu1Irq58();
    fn Ipu1Irq59();
    fn Ipu1Irq60();
    fn Ipu1Irq61();
    fn Ipu1Irq62();
    fn Ipu1Irq63();
    fn Ipu1Irq64();
    fn Ipu1Irq65();
    fn Ipu1Irq66();
    fn Ipu1Irq67();
    fn Ipu1Irq68();
    fn Ipu1Irq69();
    fn Ipu1Irq70();
    fn Ipu1Irq71();
    fn Ipu1Irq72();
    fn Ipu1Irq73();
    fn Ipu1Irq74();
    fn Ipu1Irq75();
    fn Ipu1Irq76();
    fn Ipu1Irq77();
    fn Ipu1Irq78();
    fn Ipu1Irq79();
}

// ****************************************************************************
//
// Sub-modules
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Macros
//
// ****************************************************************************

///  Macro to override a device specific interrupt handler
///
///  # Syntax
///
///  ``` ignore
///  interrupt!(
///      // Name of the interrupt
///      $Name:ident,
///
///      // Path to the interrupt handler (a function)
///      $handler:path,
///
///      // Optional, state preserved across invocations of the handler
///      state: $State:ty = $initial_state:expr,
///  );
///  ```
///
///  Where `$Name` must match the name of one of the variants of the `Interrupt`
///  enum.
///
///  The handler must have signature `fn()` is no state was associated to it;
///  otherwise its signature must be `fn(&mut $State)`.
///
/// This implementation taken from tm4c123x v0.7
#[macro_export]
macro_rules! interrupt {
    ($Name:ident, $handler:path,state: $State:ty = $initial_state:expr) => {
        #[allow(unsafe_code)]
        #[deny(private_no_mangle_fns)]
        #[no_mangle]
        pub unsafe extern "C" fn $Name() {
            static mut STATE: $State = $initial_state;
            // Compile-time check this is a valid interrupt name
            let _ = $crate::am5728::Interrupt::$Name;
            let f: fn(&mut $State) = $handler;
            f(&mut STATE)
        }
    };
    ($Name:ident, $handler:path) => {
        #[allow(unsafe_code)]
        #[deny(private_no_mangle_fns)]
        #[no_mangle]
        pub unsafe extern "C" fn $Name() {
            // Compile-time check this is a valid interrupt name
            let _ = $crate::am5728::Interrupt::$Name;
            let f: fn() = $handler;
            f()
        }
    };
}

// ****************************************************************************
//
// Public Types / Traits
//
// ****************************************************************************

/// A singleton, representing our chip.
pub struct Am5728<'a, T>
where
    T: rt::AddressMapper,
    T: 'a,
{
    mapper: &'a T,
}

#[derive(Debug, Copy, Clone)]
pub enum MailboxUser {
    User0,
    User1,
    User2,
    User3,
}

#[derive(Debug, Copy, Clone)]
pub enum MailboxSlot {
    Slot0,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    Slot10,
    Slot11,
}

#[derive(Debug, Copy, Clone)]
pub enum MailboxId {
    Mailbox1,
    Mailbox2,
    Mailbox3,
    Mailbox4,
    Mailbox5,
    Mailbox6,
    Mailbox7,
    Mailbox8,
    Mailbox9,
    Mailbox10,
    Mailbox11,
    Mailbox12,
    Mailbox13,
}

#[derive(Debug, Copy, Clone)]
pub struct MailboxLocation {
    pub id: MailboxId,
    pub user: MailboxUser,
    pub slot: MailboxSlot,
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum CacheFlushMode {
    WriteBack = CFG_MAINT_CLEAN,
    Invalidate = CFG_MAINT_INVALIDATE,
    InvalidateWriteBack = CFG_MAINT_CLEAN | CFG_MAINT_INVALIDATE,
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum CacheFlushAllMode {
    Flush = MMU_MAINT_G_FLUSH,
    WriteBack = MMU_MAINT_CLEAN,
    Invalidate = MMU_MAINT_INVALIDATE,
    InvalidateWriteBack = MMU_MAINT_INVALIDATE | MMU_MAINT_CLEAN,
}

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    /// xlate_mmu_fault (from L2 MMU)
    Ipu1Irq16,
    /// Unicache or MMU maintenance complete
    Ipu1Irq17,
    /// CTM timer event (timer #1)
    Ipu1Irq18,
    /// Semaphore interrupt (1 to each core)
    Ipu1Irq19,
    /// ICECrusher (1 to each core)
    Ipu1Irq20,
    /// Ducati imprecise fault (from interconnect)
    Ipu1Irq21,
    /// CTM timer event (timer #2)
    Ipu1Irq22,
    /// Display controller interrupt
    Ipu1Irq23,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq24,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq25,
    /// HDMI interrupt
    Ipu1Irq26,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq27,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq28,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq29,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq30,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq31,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq32,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq33,
    /// System DMA interrupt 0
    Ipu1Irq34,
    /// System DMA interrupt 1
    Ipu1Irq35,
    /// System DMA interrupt 2
    Ipu1Irq36,
    /// System DMA interrupt 3
    Ipu1Irq37,
    /// IVA mailbox user 2 interrupt
    Ipu1Irq38,
    /// IVA ICONT2 sync interrupt
    Ipu1Irq39,
    /// IVA ICONT1 sync interrupt
    Ipu1Irq40,
    /// I2C1 interrupt
    Ipu1Irq41,
    /// I2C2 interrupt
    Ipu1Irq42,
    /// I2C3 interrupt
    Ipu1Irq43,
    /// I2C4 interrupt
    Ipu1Irq44,
    /// UART3 interrupt
    Ipu1Irq45,
    /// L3_MAIN application or non-attributable error
    Ipu1Irq46,
    /// PRCM interrupt to IPU1
    Ipu1Irq47,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq48,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq49,
    /// Mailbox 1 user 2 interrupt
    Ipu1Irq50,
    /// GPIO1 interrupt 1
    Ipu1Irq51,
    /// GPIO2 interrupt 1
    Ipu1Irq52,
    /// TIMER3 interrupt
    Ipu1Irq53,
    /// TIMER4 interrupt
    Ipu1Irq54,
    /// TIMER9 interrupt
    Ipu1Irq55,
    /// TIMER11 interrupt
    Ipu1Irq56,
    /// McSPI1 interrupt
    Ipu1Irq57,
    /// McSPI2 interrupt
    Ipu1Irq58,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq59,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq60,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq61,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq62,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq63,
    /// DMM interrupt
    Ipu1Irq64,
    /// BB2D interrupt
    Ipu1Irq65,
    /// MMC1 interrupt
    Ipu1Irq66,
    /// MMC2 interrupt
    Ipu1Irq67,
    /// MMC3 interrupt
    Ipu1Irq68,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq69,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq70,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq71,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq72,
    /// USB1 interrupt 1
    Ipu1Irq73,
    /// USB2 interrupt 0
    Ipu1Irq74,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq75,
    /// USB2 interrupt 1
    Ipu1Irq76,
    /// USB3 interrupt 0
    Ipu1Irq77,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq78,
    /// Reserved by default but can be remapped to a valid interrupt source
    Ipu1Irq79,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
/// Lower numeric values have higher 'urgency'; that is, they can pre-empty
/// interrupts with higher numeric values (i.e. lower urgency). The bottom 4
/// bits of the 8-bit priority registers are ignored on this chip, so 16 is
/// the next-highest priority after zero.
pub enum InterruptPriority {
    Prio00 = 0 << 4,
    Prio01 = 1 << 4,
    Prio02 = 2 << 4,
    Prio03 = 3 << 4,
    Prio04 = 4 << 4,
    Prio05 = 5 << 4,
    Prio06 = 6 << 4,
    Prio07 = 7 << 4,
    Prio08 = 8 << 4,
    Prio09 = 9 << 4,
    Prio10 = 10 << 4,
    Prio11 = 11 << 4,
    Prio12 = 12 << 4,
    Prio13 = 13 << 4,
    Prio14 = 14 << 4,
    Prio15 = 15 << 4,
}

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

pub const L3_OCMC_RAM: usize = 0x4030_0000;
pub const L4_PERIPHERAL_L4PER1: usize = 0x4800_0000;
pub const L4_PERIPHERAL_L4PER2: usize = 0x4840_0000;
pub const L4_PERIPHERAL_L4PER3: usize = 0x4880_0000;
pub const L4_PERIPHERAL_L4CFG: usize = 0x4A00_0000;
pub const L3_PERIPHERAL_PRUSS: usize = 0x4B20_0000;
pub const L3_PERIPHERAL_DMM: usize = 0x4E00_0000;
pub const L4_PERIPHERAL_L4EMU: usize = 0x5400_0000;
pub const L3_IVAHD_CONFIG: usize = 0x5A00_0000;
pub const L3_IVAHD_SL2: usize = 0x5B00_0000;
pub const L3_TILER_MODE_0_1: usize = 0x6000_0000;
pub const L3_TILER_MODE_2: usize = 0x7000_0000;
pub const L3_TILER_MODE_3: usize = 0x7800_0000;
pub const L3_EMIF_SDRAM: usize = 0xA000_0000;

// Default IRQ mappings

pub const DEFAULT_BB2D_IRQ: Interrupt = Interrupt::Ipu1Irq65;
pub const DEFAULT_CTM_TIM_EVENT1_IRQ: Interrupt = Interrupt::Ipu1Irq18;
pub const DEFAULT_CTM_TIM_EVENT2_IRQ: Interrupt = Interrupt::Ipu1Irq22;
pub const DEFAULT_DISPC_IRQ: Interrupt = Interrupt::Ipu1Irq23;
pub const DEFAULT_DMA_SYSTEM_IRQ_0: Interrupt = Interrupt::Ipu1Irq34;
pub const DEFAULT_DMA_SYSTEM_IRQ_1: Interrupt = Interrupt::Ipu1Irq35;
pub const DEFAULT_DMA_SYSTEM_IRQ_2: Interrupt = Interrupt::Ipu1Irq36;
pub const DEFAULT_DMA_SYSTEM_IRQ_3: Interrupt = Interrupt::Ipu1Irq37;
pub const DEFAULT_DMM_IRQ: Interrupt = Interrupt::Ipu1Irq64;
pub const DEFAULT_GPIO1_IRQ_1: Interrupt = Interrupt::Ipu1Irq51;
pub const DEFAULT_GPIO2_IRQ_1: Interrupt = Interrupt::Ipu1Irq52;
pub const DEFAULT_HDMI_IRQ: Interrupt = Interrupt::Ipu1Irq26;
pub const DEFAULT_HWSEM_M4_IRQ: Interrupt = Interrupt::Ipu1Irq19;
pub const DEFAULT_I2C1_IRQ: Interrupt = Interrupt::Ipu1Irq41;
pub const DEFAULT_I2C2_IRQ: Interrupt = Interrupt::Ipu1Irq42;
pub const DEFAULT_I2C3_IRQ: Interrupt = Interrupt::Ipu1Irq43;
pub const DEFAULT_I2C4_IRQ: Interrupt = Interrupt::Ipu1Irq44;
pub const DEFAULT_ICE_NEMU_IRQ: Interrupt = Interrupt::Ipu1Irq20;
pub const DEFAULT_IPU_IMP_FAULT_IRQ: Interrupt = Interrupt::Ipu1Irq21;
pub const DEFAULT_IVA_IRQ_MAILBOX_2: Interrupt = Interrupt::Ipu1Irq38;
pub const DEFAULT_IVA_IRQ_SYNC_0: Interrupt = Interrupt::Ipu1Irq40;
pub const DEFAULT_IVA_IRQ_SYNC_1: Interrupt = Interrupt::Ipu1Irq39;
pub const DEFAULT_L3_MAIN_IRQ_APP_ERR: Interrupt = Interrupt::Ipu1Irq46;
pub const DEFAULT_MAILBOX1_IRQ_USER2: Interrupt = Interrupt::Ipu1Irq50;
pub const DEFAULT_MCSPI1_IRQ: Interrupt = Interrupt::Ipu1Irq57;
pub const DEFAULT_MCSPI2_IRQ: Interrupt = Interrupt::Ipu1Irq58;
pub const DEFAULT_MMC1_IRQ: Interrupt = Interrupt::Ipu1Irq66;
pub const DEFAULT_MMC2_IRQ: Interrupt = Interrupt::Ipu1Irq67;
pub const DEFAULT_MMC3_IRQ: Interrupt = Interrupt::Ipu1Irq68;
pub const DEFAULT_PRM_IRQ_IPU1: Interrupt = Interrupt::Ipu1Irq47;
pub const DEFAULT_TIMER11_IRQ: Interrupt = Interrupt::Ipu1Irq56;
pub const DEFAULT_TIMER3_IRQ: Interrupt = Interrupt::Ipu1Irq53;
pub const DEFAULT_TIMER4_IRQ: Interrupt = Interrupt::Ipu1Irq54;
pub const DEFAULT_TIMER9_IRQ: Interrupt = Interrupt::Ipu1Irq55;
pub const DEFAULT_UART3_IRQ: Interrupt = Interrupt::Ipu1Irq45;
pub const DEFAULT_UNICACHE_MMU_IRQ: Interrupt = Interrupt::Ipu1Irq17;
pub const DEFAULT_USB1_IRQ_INTR1: Interrupt = Interrupt::Ipu1Irq73;
pub const DEFAULT_USB2_IRQ_INTR0: Interrupt = Interrupt::Ipu1Irq74;
pub const DEFAULT_USB2_IRQ_INTR1: Interrupt = Interrupt::Ipu1Irq76;
pub const DEFAULT_USB3_IRQ_INTR0: Interrupt = Interrupt::Ipu1Irq77;
pub const DEFAULT_XLATE_MMU_FAULT_IRQ: Interrupt = Interrupt::Ipu1Irq16;

#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [unsafe extern "C" fn(); 64] = [
    Ipu1Irq16, Ipu1Irq17, Ipu1Irq18, Ipu1Irq19, Ipu1Irq20, Ipu1Irq21, Ipu1Irq22, Ipu1Irq23,
    Ipu1Irq24, Ipu1Irq25, Ipu1Irq26, Ipu1Irq27, Ipu1Irq28, Ipu1Irq29, Ipu1Irq30, Ipu1Irq31,
    Ipu1Irq32, Ipu1Irq33, Ipu1Irq34, Ipu1Irq35, Ipu1Irq36, Ipu1Irq37, Ipu1Irq38, Ipu1Irq39,
    Ipu1Irq40, Ipu1Irq41, Ipu1Irq42, Ipu1Irq43, Ipu1Irq44, Ipu1Irq45, Ipu1Irq46, Ipu1Irq47,
    Ipu1Irq48, Ipu1Irq49, Ipu1Irq50, Ipu1Irq51, Ipu1Irq52, Ipu1Irq53, Ipu1Irq54, Ipu1Irq55,
    Ipu1Irq56, Ipu1Irq57, Ipu1Irq58, Ipu1Irq59, Ipu1Irq60, Ipu1Irq61, Ipu1Irq62, Ipu1Irq63,
    Ipu1Irq64, Ipu1Irq65, Ipu1Irq66, Ipu1Irq67, Ipu1Irq68, Ipu1Irq69, Ipu1Irq70, Ipu1Irq71,
    Ipu1Irq72, Ipu1Irq73, Ipu1Irq74, Ipu1Irq75, Ipu1Irq76, Ipu1Irq77, Ipu1Irq78, Ipu1Irq79,
];

// ****************************************************************************
//
// Private Types / Traits
//
// ****************************************************************************

/// Corresponds to IPUx_UNICACHE_CFG in section 7.4.2
#[repr(C)]
struct UnicacheConfig {
    info: RW<u32>,
    config: RW<u32>,
    int: RW<u32>,
    ocp: RW<u32>,
    maint: RW<u32>,
    mt_start: RW<usize>,
    mt_end: RW<usize>,
    ct_addr: RW<usize>,
    ct_data: RW<u32>,
}

#[repr(C)]
struct UnicacheMmu {
    // large pages (4 out of 8)
    large_addr: [RW<usize>; 4],
    _padding: [usize; 4],

    large_xlte: [RW<usize>; 4],
    _padding2: [usize; 4],

    large_policy: [RW<u32>; 4],
    _padding3: [usize; 4],

    // medium pages (2 out of 16)
    medium_addr: [RW<usize>; 2],
    _padding4: [usize; 14],

    medium_xlte: [RW<usize>; 2],
    _padding5: [usize; 14],

    medium_policy: [RW<u32>; 2],
    _padding6: [usize; 14],

    // small pages (10 out of 32)
    small_addr: [RW<usize>; 10],
    _padding7: [usize; 22],

    small_xlte: [RW<usize>; 10],
    _padding8: [usize; 22],

    small_policy: [RW<u32>; 10],
    _padding9: [usize; 22],

    small_maint: [RW<u32>; 10],
    _padding10: [usize; 22],

    // lines
    line_addr: [RW<usize>; 32],
    line_xlte: [RW<usize>; 32],
    line_policy: [RW<u32>; 32],

    // debug
    debug_xlte: RW<usize>,
    debug_policy: RW<u32>,

    // maintenance
    maint: RW<u32>,
    mstart: RW<usize>,
    mend: RW<usize>,
    maint_status: RW<u32>,
    mmu_config: RW<u32>,
}

#[repr(C)]
#[allow(dead_code)]
pub struct MailboxIrq {
    status_raw: RW<u32>,
    status_clr: RW<u32>,
    enable_set: RW<u32>,
    enable_clr: RW<u32>,
}

#[repr(C)]
#[allow(dead_code)]
pub struct Mailbox {
    /// Address Offset 0x0000_0000
    revision: RO<u32>,
    _padding: [u32; 3],
    /// Address Offset 0x0000_0010
    sysconfig: RW<u32>,
    _padding2: [u32; 11],
    /// Address Offset 0x0000_0040
    message: [RW<u32>; 12],
    _padding3: [u32; 4],
    /// Address Offset 0x0000_0080
    fifo_status: [RW<u32>; 12],
    _padding4: [u32; 4],
    /// Address Offset 0x0000_00C0
    msg_status: [RW<u32>; 12],
    _padding5: [u32; 4],
    /// Address Offset 0x0000_0100
    irq: [MailboxIrq; 4],
    /// Address Offset 0x0000_0140
    irq_eoi: RW<u32>,
}

/// The Wake-Up Generator. Allows our core to come out of idle.
/// Corresponds to IPUx_WUGEN in section 7.4.7
#[repr(C)]
struct WuGen {
    /// Used to interrup the other core
    cortexm4_ctrl: RW<u32>,
    /// Set the standby protocol
    standby_core_sysconfig: RW<u32>,
    /// Set the idle protocol
    idle_core_sysconfig: RW<u32>,
    /// Interrupt mask for interrupts 0-31
    wugen_evt0: RW<u32>,
    /// Interrupt mask for interrupts 32-63
    wugen_evt1: RW<u32>,
    _reserved: RW<u32>,
}

/// Stores two 9-bit values with 16-bit alignment
#[repr(C)]
struct IrqRegister {
    field: RW<u32>,
}

/// Controls the Crossbar IRQ mapping for IPU1. Use these registers to send a
/// Crossbar IRQ into the IPU on the given interrupt line.
#[repr(C)]
struct CtrlCoreIpu1 {
    irq_23_24: IrqRegister,
    irq_25_26: IrqRegister,
    irq_27_28: IrqRegister,
    irq_29_30: IrqRegister,
    irq_31_32: IrqRegister,
    irq_33_34: IrqRegister,
    irq_35_36: IrqRegister,
    irq_37_38: IrqRegister,
    irq_39_40: IrqRegister,
    irq_41_42: IrqRegister,
    irq_43_44: IrqRegister,
    irq_45_46: IrqRegister,
    irq_47_48: IrqRegister,
    irq_49_50: IrqRegister,
    irq_51_52: IrqRegister,
    irq_53_54: IrqRegister,
    irq_55_56: IrqRegister,
    irq_57_58: IrqRegister,
    irq_59_60: IrqRegister,
    irq_61_62: IrqRegister,
    irq_63_64: IrqRegister,
    irq_65_66: IrqRegister,
    irq_67_68: IrqRegister,
    irq_69_70: IrqRegister,
    irq_71_72: IrqRegister,
    irq_73_74: IrqRegister,
    irq_75_76: IrqRegister,
    irq_77_78: IrqRegister,
    irq_79_80: IrqRegister,
}

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

/// This is the mapped address as the HW maps 0x4000_0000 to 0x5508_0000 by
/// default. See Table 7-64. This is a Cortex-M4 device address, so it doesn't
/// need mapping through the resource table.
const UNICACHE_CFG_ADDR: usize = 0x4000_0000;

/// This is the mapped address as the HW maps 0x4000_0000 to 0x5508_0000 by
/// default. See Table 7-64. This is a Cortex-M4 device address, so it doesn't
/// need mapping through the resource table.
const WUGEN_ADDR: usize = 0x4000_1000;

const CFG_CONFIG_UNLOCK_MAIN: u32 = 1 << 4;
const CFG_CONFIG_UNLOCK_PORT: u32 = 1 << 3;
const CFG_CONFIG_UNLOCK_INT: u32 = 1 << 2;
const CFG_CONFIG_DISABLE_BYPASS: u32 = 1 << 1;
const CFG_CONFIG_CACHE_LOCK: u32 = 1 << 0;

const CFG_OCP_CLEANBUF: u32 = 1 << 5;
const CFG_OCP_PREFETCH: u32 = 1 << 4;
const CFG_OCP_CACHED: u32 = 1 << 3;
const CFG_OCP_WRALLOCATE: u32 = 1 << 2;
const CFG_OCP_WRBUFFER: u32 = 1 << 1;
const CFG_OCP_WRAP: u32 = 1 << 0;

const CFG_MAINT_INTERRUPT: u32 = 1 << 5;
const CFG_MAINT_INVALIDATE: u32 = 1 << 4;
const CFG_MAINT_CLEAN: u32 = 1 << 3;
const CFG_MAINT_UNLOCK: u32 = 1 << 2;
const CFG_MAINT_LOCK: u32 = 1 << 1;
const CFG_MAINT_PRELOAD: u32 = 1 << 0;

const CFG_INT_READ: u32 = 1 << 4;
const CFG_INT_WRITE: u32 = 1 << 3;
const CFG_INT_MAINT: u32 = 1 << 2;
const CFG_INT_PAGEFAULT: u32 = 1 << 1;
const CFG_INT_CONFIG: u32 = 1 << 0;

/// This is the mapped address as the HW maps 0x4000_0000 to 0x5508_0000 by
/// default. See Table 7-64. This is a Cortex-M4 device address, so it doesn't
/// need mapping through the resource table.
const UNICACHE_MMU_ADDR: usize = 0x4000_0800;

const MMU_POLICY_ENABLED: u32 = (1 << 0);
const MMU_POLICY_LARGE: u32 = (1 << 1);
const MMU_POLICY_CACHEABLE: u32 = (1 << 16);
const MMU_POLICY_POSTED: u32 = (1 << 17);
const MMU_POLICY_ALLOCATE: u32 = (1 << 18);
const MMU_POLICY_WRITE_BACK: u32 = (1 << 19);

const MMU_MAINT_G_FLUSH: u32 = 1 << 10;
const MMU_MAINT_L1_CACHE1: u32 = 1 << 7;
const MMU_MAINT_CPU_INTERRUPT: u32 = 1 << 6;
const MMU_MAINT_HOST_INTERRUPT: u32 = 1 << 5;
const MMU_MAINT_INVALIDATE: u32 = 1 << 4;
const MMU_MAINT_CLEAN: u32 = 1 << 3;
const MMU_MAINT_UNLOCK: u32 = 1 << 2;
const MMU_MAINT_LOCK: u32 = 1 << 1;
const MMU_MAINT_PRELOAD: u32 = 1 << 0;

const MMU_CONFIG_PRIVILEGE: u32 = 1 << 1;
const MMU_CONFIG_MMU_LOCK: u32 = 1 << 0;

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

impl<'a, T> Am5728<'a, T>
where
    T: rt::AddressMapper,
{
    /// Returns an object the first time, None the second time.
    pub fn claim(mapper: &'a T) -> Option<Am5728<'a, T>> {
        static mut AVAILABLE: bool = true;
        unsafe {
            if AVAILABLE {
                let mut r = Am5728 { mapper };
                r.setup();
                AVAILABLE = false;
                Some(r)
            } else {
                None
            }
        }
    }

    /// Configure and enable the L1 'Unicache' and enable interrupts. Unsafe as
    /// you must only call this function once.
    pub unsafe fn setup(&mut self) {
        cortex_m::interrupt::disable();
        unicache_mmu_setup();
        let unicache_cfg = get_unicache_config();
        unicache_cfg
            .config
            .write(CFG_CONFIG_UNLOCK_MAIN | CFG_CONFIG_UNLOCK_PORT | CFG_CONFIG_UNLOCK_INT);
        // OCP_CACHED is on by default. Turn it off to disable caching.
        unicache_cfg.ocp.write(0x0000_0000);

        // This is what the TI code does but it doesn't make any sense.
        while (unicache_cfg.maint.read() & 0x1F) != 0 {
            cortex_m::asm::nop();
        }
        self.cache_flush_all(CacheFlushAllMode::Flush);
        self.cache_enable();

        // Need to enable interrupt for non-empty and not for non-full!

        let core = CtrlCoreIpu1::get(self.mapper).expect("CtrlCoreIpu1 bad RT");
        // Crossbar 250 is Mailbox 5 User 1

        // Disable all the interrupts!
        core.irq_23_24.set_lower(0);
        core.irq_23_24.set_higher(0);
        core.irq_25_26.set_lower(0);
        core.irq_25_26.set_higher(0);
        core.irq_27_28.set_lower(0);
        core.irq_27_28.set_higher(0);
        core.irq_29_30.set_lower(0);
        core.irq_29_30.set_higher(0);
        core.irq_31_32.set_lower(0);
        core.irq_31_32.set_higher(0);
        core.irq_33_34.set_lower(0);
        core.irq_33_34.set_higher(0);
        core.irq_35_36.set_lower(0);
        core.irq_35_36.set_higher(0);
        core.irq_37_38.set_lower(0);
        core.irq_37_38.set_higher(0);
        core.irq_39_40.set_lower(0);
        core.irq_39_40.set_higher(0);
        core.irq_41_42.set_lower(0);
        core.irq_41_42.set_higher(0);
        core.irq_43_44.set_lower(0);
        core.irq_43_44.set_higher(250);
        core.irq_45_46.set_lower(0);
        core.irq_45_46.set_higher(0);
        core.irq_47_48.set_lower(0);
        core.irq_47_48.set_higher(0);
        core.irq_49_50.set_lower(0);
        core.irq_49_50.set_higher(0);
        core.irq_51_52.set_lower(0);
        core.irq_51_52.set_higher(0);
        core.irq_53_54.set_lower(0);
        core.irq_53_54.set_higher(0);
        core.irq_55_56.set_lower(0);
        core.irq_55_56.set_higher(0);
        core.irq_57_58.set_lower(0);
        core.irq_57_58.set_higher(0);
        core.irq_59_60.set_lower(0);
        core.irq_59_60.set_higher(0);
        core.irq_61_62.set_lower(0);
        core.irq_61_62.set_higher(0);
        core.irq_63_64.set_lower(0);
        core.irq_63_64.set_higher(0);
        core.irq_65_66.set_lower(0);
        core.irq_65_66.set_higher(0);
        core.irq_67_68.set_lower(0);
        core.irq_67_68.set_higher(0);
        core.irq_69_70.set_lower(0);
        core.irq_69_70.set_higher(0);
        core.irq_71_72.set_lower(0);
        core.irq_71_72.set_higher(0);
        core.irq_73_74.set_lower(0);
        core.irq_73_74.set_higher(0);
        core.irq_75_76.set_lower(0);
        core.irq_75_76.set_higher(0);
        core.irq_77_78.set_lower(0);
        core.irq_77_78.set_higher(0);
        core.irq_79_80.set_lower(0);
        core.irq_79_80.set_higher(0);

        // // Enable mailbox interrupts
        // let wugen = get_wugen();
        // wugen.wake_on_interrupt(Interrupt::Ipu1Irq44);

        self.interrupt_disable(Interrupt::Ipu1Irq44);
    }

    /// Enable the L1 'Unicache'
    pub fn cache_enable(&mut self) {
        unsafe {
            let unicache_cfg = get_unicache_config();
            // Turn the cache on
            unicache_cfg
                .config
                .modify(|w| w | CFG_CONFIG_DISABLE_BYPASS);
            // Ensure write is complete
            let _ = unicache_cfg.config.read();
        }
    }

    /// Disable the L1 'Unicache'
    pub fn cache_disable(&mut self) {
        unsafe {
            let unicache_cfg = get_unicache_config();
            // Turn the cache on
            unicache_cfg
                .config
                .modify(|w| w & !CFG_CONFIG_DISABLE_BYPASS);
            // Ensure write is complete
            let _ = unicache_cfg.config.read();
        }
    }

    /// Cache flush everything using the Unicache AMMU.
    ///
    /// It's unclear what the difference is between flushing the L1 Cache
    /// (through UnicacheConfig) and flushing the AMMU (through UnicacheMmu).
    /// The TI code uses the former for small regions and the latter only with
    /// 0x0000_0000/0xFFFF_FFFF. Maybe it's a performance thing?
    pub fn cache_flush_all(&mut self, mode: CacheFlushAllMode) {
        let unicache_mmu = get_unicache_mmu();
        unsafe {
            unicache_mmu.mstart.write(0x00000000);
            unicache_mmu.mend.write(0xffffffff);
            unicache_mmu.maint.modify(|w| w | (mode as u32));
            while (unicache_mmu.maint.read() & (mode as u32)) != 0 {
                cortex_m::asm::nop();
            }
        }
    }

    /// Tell the unicache to invalidate/writeback a specific object from the
    /// L1 cache.
    pub fn cache_flush<M>(&mut self, obj: &M, len: usize, mode: CacheFlushMode) {
        unsafe {
            let address = obj as *const _ as usize;
            self.cache_flush_address(address, len, mode);
        }
    }

    /// Tell the unicache to invalidate/writeback a specific address range
    /// from the L1 cache.
    pub unsafe fn cache_flush_address(&mut self, address: usize, len: usize, mode: CacheFlushMode) {
        let unicache_cfg = get_unicache_config();
        unicache_cfg.mt_start.write(address);
        unicache_cfg.mt_end.write(address + len - 1);
        let mode: u32 = mode as u32;
        unicache_cfg.maint.modify(|v| v | mode);
        while (unicache_cfg.maint.read() & 0x1f) != 0 {
            cortex_m::asm::nop();
        }
    }

    /// Send a message to the host.
    ///
    /// This invoves processor 6 talking to processor 8.
    pub fn send_message(&mut self, id: u32, location: MailboxLocation) {
        let mailbox = get_mailbox(location.id, self.mapper).expect("Bad resource_table");
        while mailbox.msg_status[location.slot as usize].read() != 0 {
            // spin
        }
        unsafe {
            mailbox.message[location.slot as usize].write(id);
        }
    }

    /// Get any message the host may have for us.
    ///
    /// This invoves processor 8 talking to processor 6.
    pub fn get_message(&mut self, location: MailboxLocation) -> Option<u32>
    where
        T: rt::AddressMapper,
    {
        let mailbox = get_mailbox(location.id, self.mapper).expect("Bad resource_table");
        match mailbox.get_message(location.slot) {
            Some(m) => Some(m),
            None => {
                // mailbox.clear_data_interrupt(location.user, location.slot);
                None
            }
        }
    }

    /// Enable an interrupt on IPU1_C0
    pub fn interrupt_enable(&mut self, interrupt: Interrupt) {
        let mut peripherals = unsafe { cortex_m::Peripherals::steal() };
        peripherals.NVIC.enable(interrupt);
    }

    /// Clear a pending interrupt on IPU1_C0
    pub fn interrupt_clear(&mut self, interrupt: Interrupt) {
        let mut peripherals = unsafe { cortex_m::Peripherals::steal() };
        peripherals.NVIC.clear_pending(interrupt);
    }

    /// Disable an interrupt on IPU1_C0
    pub fn interrupt_disable(&mut self, interrupt: Interrupt) {
        let mut peripherals = unsafe { cortex_m::Peripherals::steal() };
        peripherals.NVIC.disable(interrupt);
    }

    /// Set interrupt priority on IPU1_C0
    pub fn interrupt_priority_set(&mut self, interrupt: Interrupt, priority: InterruptPriority) {
        let mut peripherals = unsafe { cortex_m::Peripherals::steal() };
        unsafe { peripherals.NVIC.set_priority(interrupt, priority as u8) };
    }

    pub fn enable_mailbox_data_interrupt(&mut self, location: MailboxLocation) {
        let mailbox = get_mailbox(location.id, self.mapper).expect("Bad resource_table");
        mailbox.enable_data_interrupt(location.user, location.slot);
    }

    pub fn disable_mailbox_interrupts(&mut self, id: MailboxId, user: MailboxUser) {
        let mailbox = get_mailbox(id, self.mapper).expect("Bad resource_table");
        mailbox.disable_interrupts(user);
    }
}

impl Mailbox {
    pub fn get_message(&mut self, slot: MailboxSlot) -> Option<u32> {
        // As per Section 19.4.1.3.2
        if self.msg_status[slot as usize].read() != 0 {
            let msg = self.message[slot as usize].read();
            Some(msg)
        } else {
            None
        }
    }

    /// Mark an interrupt as handled
    pub fn clear_interrupts(&mut self, user: MailboxUser) {
        unsafe {
            self.irq[user as usize].status_clr.write(0xFFFFFFFF);
        }
    }

    /// Enable interrupt from the Mailbox on data received
    pub fn enable_data_interrupt(&mut self, user: MailboxUser, slot: MailboxSlot) {
        unsafe {
            // Clear all interrupts
            self.irq[user as usize].status_clr.write(0xFFFFFFFF);
            self.irq[user as usize].enable_clr.write(0xFFFFFFFF);
            // We want to know when there's data on this specific slot
            self.irq[user as usize]
                .enable_set
                .write(slot.get_data_bit());
        }
    }

    /// Disable interrupts from the Mailbox
    pub fn disable_interrupts(&mut self, user: MailboxUser) {
        unsafe {
            self.irq[user as usize].status_clr.write(0xFFFFFFFF);
            self.irq[user as usize].enable_clr.write(0xFFFFFFFF);
        }
    }

    pub fn get_raw(&self, user: MailboxUser) -> u32 {
        self.irq[user as usize].status_raw.read()
    }

    pub fn get_masked(&self, user: MailboxUser) -> u32 {
        self.irq[user as usize].status_clr.read()
    }
}

impl MailboxSlot {
    pub fn get_data_bit(&self) -> u32 {
        1 << ((*self as u32) * 2)
    }

    pub fn get_space_bit(&self) -> u32 {
        1 << (((*self as u32) + 1) * 2)
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

/// Configure the MMU paging. We basically make it transparent.
fn unicache_mmu_setup() {
    // This maps 512 MiB from 0x0000_0000 with no translation.
    // This region contains machine-code and is cacheable.
    unicache_mmu_configure_large_page(
        0,
        0x00000000,
        0xFFFFFFFF,
        MMU_POLICY_POSTED | MMU_POLICY_CACHEABLE | MMU_POLICY_LARGE | MMU_POLICY_ENABLED,
    );
    // These next three map in 1.5 GiB from 0x6000_0000 to 0xC000_0000 with no translation.
    // This region contains peripherals. It is non-cacheable.
    unicache_mmu_configure_large_page(
        1,
        0x60000000,
        0xFFFFFFFF,
        MMU_POLICY_POSTED | MMU_POLICY_LARGE | MMU_POLICY_ENABLED,
    );
    // This region contains shared memory and IPC data. It is cacheable.
    unicache_mmu_configure_large_page(
        2,
        0x80000000,
        0xFFFFFFFF,
        MMU_POLICY_POSTED | MMU_POLICY_CACHEABLE | MMU_POLICY_LARGE | MMU_POLICY_ENABLED,
    );
    // This region is for DMM and TILER. It is cacheable.
    unicache_mmu_configure_large_page(
        3,
        0xa0000000,
        0xFFFFFFFF,
        MMU_POLICY_POSTED | MMU_POLICY_CACHEABLE | MMU_POLICY_LARGE | MMU_POLICY_ENABLED,
    );
    // There are two small pages mapped by the hardware. 0x????_???? to
    // 0x5502_0000 and 0x4000_0000 to 0x5508_0000. We need to make the latter
    // of these larger as it defaults to 4 KiB, which doesn't cover the inter-
    // core interrupt peripheral.
    unicache_mmu_configure_small_page(
        1,
        0x40000000,
        0x55080000,
        MMU_POLICY_LARGE | MMU_POLICY_ENABLED,
    );
    // Map 64 KiB of L2RAM to 0x2000_0000 using four 16 KiB pages.
    unicache_mmu_configure_small_page(
        2,
        0x20000000,
        0x55020000,
        MMU_POLICY_POSTED
            | MMU_POLICY_CACHEABLE
            | MMU_POLICY_LARGE
            | MMU_POLICY_ALLOCATE
            | MMU_POLICY_WRITE_BACK
            | MMU_POLICY_ENABLED,
    );
    unicache_mmu_configure_small_page(
        3,
        0x20004000,
        0x55024000,
        MMU_POLICY_POSTED
            | MMU_POLICY_CACHEABLE
            | MMU_POLICY_LARGE
            | MMU_POLICY_ALLOCATE
            | MMU_POLICY_WRITE_BACK
            | MMU_POLICY_ENABLED,
    );
    unicache_mmu_configure_small_page(
        4,
        0x20008000,
        0x55028000,
        MMU_POLICY_POSTED
            | MMU_POLICY_CACHEABLE
            | MMU_POLICY_LARGE
            | MMU_POLICY_ALLOCATE
            | MMU_POLICY_WRITE_BACK
            | MMU_POLICY_ENABLED,
    );
    unicache_mmu_configure_small_page(
        5,
        0x2000C000,
        0x5502C000,
        MMU_POLICY_POSTED
            | MMU_POLICY_CACHEABLE
            | MMU_POLICY_LARGE
            | MMU_POLICY_ALLOCATE
            | MMU_POLICY_WRITE_BACK
            | MMU_POLICY_ENABLED,
    );
}

/// Configure a large MMU page.
fn unicache_mmu_configure_large_page(idx: usize, addr: usize, xlte: usize, policy: u32) {
    unsafe {
        let unicache_mmu = get_unicache_mmu();
        unicache_mmu.large_addr[idx].write(addr);
        unicache_mmu.large_xlte[idx].write(xlte);
        unicache_mmu.large_policy[idx].write(policy);
    }
}

/// Configure a small MMU page.
fn unicache_mmu_configure_small_page(idx: usize, addr: usize, xlte: usize, policy: u32) {
    unsafe {
        let unicache_mmu = get_unicache_mmu();
        unicache_mmu.small_addr[idx].write(addr);
        unicache_mmu.small_xlte[idx].write(xlte);
        unicache_mmu.small_policy[idx].write(policy);
    }
}

/// Get a reference to the Unicache MMU peripheral. This is local to the M4 so
/// does not need mapping.
fn get_unicache_mmu() -> &'static mut UnicacheMmu {
    unsafe { &mut *(UNICACHE_MMU_ADDR as *mut UnicacheMmu) }
}

/// Get a reference to the Unicache config peripheral. This is local to the M4
/// so does not need mapping.
fn get_unicache_config() -> &'static mut UnicacheConfig {
    unsafe { &mut *(UNICACHE_CFG_ADDR as *mut UnicacheConfig) }
}

/// Get a reference to the WuGen peripheral. This is local to the M4
/// so does not need mapping.
fn get_wugen() -> &'static mut WuGen {
    unsafe { &mut *(WUGEN_ADDR as *mut WuGen) }
}

/// Get a reference to a specific mailbox instance. The mailboxes are remote
/// to the M4 so we need something that can map the fixed physical address of
/// the peripheral to the device address we need as mapped in the IOMMU.
pub fn get_mailbox<T>(mbox_type: MailboxId, mapper: &T) -> Option<&'static mut Mailbox>
where
    T: rt::AddressMapper,
{
    // See Table 19-24 in the TRM, Mailbox Instance Summary
    // These addresses are in L4_PERIPHERAL_L4PER3 or L4_PERIPHERAL_L4CFG.
    let pa = match mbox_type {
        MailboxId::Mailbox1 => 0x4A0F_4000,
        MailboxId::Mailbox2 => 0x4883_a000,
        MailboxId::Mailbox3 => 0x4883_c000,
        MailboxId::Mailbox4 => 0x4883_e000,
        MailboxId::Mailbox5 => 0x4884_0000,
        MailboxId::Mailbox6 => 0x4884_2000,
        MailboxId::Mailbox7 => 0x4884_4000,
        MailboxId::Mailbox8 => 0x4884_6000,
        MailboxId::Mailbox9 => 0x4885_e000,
        MailboxId::Mailbox10 => 0x4886_0000,
        MailboxId::Mailbox11 => 0x4886_2000,
        MailboxId::Mailbox12 => 0x4886_4000,
        MailboxId::Mailbox13 => 0x4880_2000,
    };
    match mapper.pa_to_da(pa) {
        Some(pa) => unsafe { Some(&mut *(pa as *mut Mailbox)) },
        None => None,
    }
}

impl WuGen {
    /// Enable a wake-up trigger on IPU1 when this interrupt fires.
    pub fn wake_on_interrupt(&mut self, interrupt: Interrupt) {
        let num = interrupt.nr() as u32;
        if num >= 48 {
            // Interrupt bit is in second register of 32..63
            unsafe {
                self.wugen_evt1.modify(|m| m | num);
            }
        } else if num >= 16 {
            // Interrupt bit is in first register of 0..31
            unsafe {
                self.wugen_evt0.modify(|m| m | num);
            }
        } else {
            panic!("Can't enable WuGen interrupt when < 16")
        }
    }
}

impl IrqRegister {
    fn set_higher(&mut self, crossbar_irq: u16) {
        unsafe {
            self.field.modify(|mut w| {
                w &= 0x0000_FFFF;
                w |= (crossbar_irq as u32) << 16;
                w
            });
        }
    }

    fn set_lower(&mut self, crossbar_irq: u16) {
        unsafe {
            self.field.modify(|mut w| {
                w &= 0xFFFF_0000;
                w |= (crossbar_irq as u32) << 0;
                w
            });
        }
    }

    fn get_higher(&mut self) -> u16 {
        (self.field.read() >> 16) as u16
    }

    fn get_lower(&mut self) -> u16 {
        (self.field.read() >> 0) as u16
    }
}

impl CtrlCoreIpu1 {
    /// Get a reference to a specific IPU1 IRQ mapping instance. The peripheral is remote
    /// to the M4 so we need something that can map the fixed physical address of
    /// the peripheral to the device address we need as mapped in the IOMMU.
    fn get<T>(mapper: &T) -> Option<&'static mut CtrlCoreIpu1>
    where
        T: rt::AddressMapper,
    {
        // See Table 18-28. CTRL_MODULE_CORE Registers Mapping Summary
        match mapper.pa_to_da(0x4A00_27E0) {
            Some(pa) => unsafe { Some(&mut *(pa as *mut CtrlCoreIpu1)) },
            None => None,
        }
    }
}

unsafe impl Nr for Interrupt {
    fn nr(&self) -> u8 {
        // The first user interrupt must have the value 0 for this API, and
        // there are 16 system exceptions.
        match *self {
            Interrupt::Ipu1Irq16 => 16 - 16,
            Interrupt::Ipu1Irq17 => 17 - 16,
            Interrupt::Ipu1Irq18 => 18 - 16,
            Interrupt::Ipu1Irq19 => 19 - 16,
            Interrupt::Ipu1Irq20 => 20 - 16,
            Interrupt::Ipu1Irq21 => 21 - 16,
            Interrupt::Ipu1Irq22 => 22 - 16,
            Interrupt::Ipu1Irq23 => 23 - 16,
            Interrupt::Ipu1Irq24 => 24 - 16,
            Interrupt::Ipu1Irq25 => 25 - 16,
            Interrupt::Ipu1Irq26 => 26 - 16,
            Interrupt::Ipu1Irq27 => 27 - 16,
            Interrupt::Ipu1Irq28 => 28 - 16,
            Interrupt::Ipu1Irq29 => 29 - 16,
            Interrupt::Ipu1Irq30 => 30 - 16,
            Interrupt::Ipu1Irq31 => 31 - 16,
            Interrupt::Ipu1Irq32 => 32 - 16,
            Interrupt::Ipu1Irq33 => 33 - 16,
            Interrupt::Ipu1Irq34 => 34 - 16,
            Interrupt::Ipu1Irq35 => 35 - 16,
            Interrupt::Ipu1Irq36 => 36 - 16,
            Interrupt::Ipu1Irq37 => 37 - 16,
            Interrupt::Ipu1Irq38 => 38 - 16,
            Interrupt::Ipu1Irq39 => 39 - 16,
            Interrupt::Ipu1Irq40 => 40 - 16,
            Interrupt::Ipu1Irq41 => 41 - 16,
            Interrupt::Ipu1Irq42 => 42 - 16,
            Interrupt::Ipu1Irq43 => 43 - 16,
            Interrupt::Ipu1Irq44 => 44 - 16,
            Interrupt::Ipu1Irq45 => 45 - 16,
            Interrupt::Ipu1Irq46 => 46 - 16,
            Interrupt::Ipu1Irq47 => 47 - 16,
            Interrupt::Ipu1Irq48 => 48 - 16,
            Interrupt::Ipu1Irq49 => 49 - 16,
            Interrupt::Ipu1Irq50 => 50 - 16,
            Interrupt::Ipu1Irq51 => 51 - 16,
            Interrupt::Ipu1Irq52 => 52 - 16,
            Interrupt::Ipu1Irq53 => 53 - 16,
            Interrupt::Ipu1Irq54 => 54 - 16,
            Interrupt::Ipu1Irq55 => 55 - 16,
            Interrupt::Ipu1Irq56 => 56 - 16,
            Interrupt::Ipu1Irq57 => 57 - 16,
            Interrupt::Ipu1Irq58 => 58 - 16,
            Interrupt::Ipu1Irq59 => 59 - 16,
            Interrupt::Ipu1Irq60 => 60 - 16,
            Interrupt::Ipu1Irq61 => 61 - 16,
            Interrupt::Ipu1Irq62 => 62 - 16,
            Interrupt::Ipu1Irq63 => 63 - 16,
            Interrupt::Ipu1Irq64 => 64 - 16,
            Interrupt::Ipu1Irq65 => 65 - 16,
            Interrupt::Ipu1Irq66 => 66 - 16,
            Interrupt::Ipu1Irq67 => 67 - 16,
            Interrupt::Ipu1Irq68 => 68 - 16,
            Interrupt::Ipu1Irq69 => 69 - 16,
            Interrupt::Ipu1Irq70 => 70 - 16,
            Interrupt::Ipu1Irq71 => 71 - 16,
            Interrupt::Ipu1Irq72 => 72 - 16,
            Interrupt::Ipu1Irq73 => 73 - 16,
            Interrupt::Ipu1Irq74 => 74 - 16,
            Interrupt::Ipu1Irq75 => 75 - 16,
            Interrupt::Ipu1Irq76 => 76 - 16,
            Interrupt::Ipu1Irq77 => 77 - 16,
            Interrupt::Ipu1Irq78 => 78 - 16,
            Interrupt::Ipu1Irq79 => 79 - 16,
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
