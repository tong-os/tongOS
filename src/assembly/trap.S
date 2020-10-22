# trap.S
# Trap handler and global context
# Steve Operating System
# Stephen Marz
# tongOs team

.section .text.tongos

.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set REG_SIZE, 8   # Register size (in bytes)

# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=t6
    sd  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
    ld  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
    fsd f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
    fld f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm

.global __tong_os_trap
__tong_os_trap:
    # Read trap frame
    csrrw a0, mscratch, a0

    la sp, _stack_end

    call tong_os_trap


.global __tong_os_switch_to_user
__tong_os_switch_to_user:
    # a0 = trapFrame
    # a1 = pc
    # a2 = satp
    csrw   mscratch, a0

    # Load program counter
    ld    a1, 65*8(a0)
    # Load satp
    ld    a2, 64*8(a0)
    # Load processor mode
    ld    a3, 69*8(a0)
    # Pid
    # ld    a4, 544(a0)

    # 1 << 7 is MPIE
    # Since user mode is 00, we don't need to set anything
    # in MPP (bits 12:11)
    li    t0, 1 << 7 | 1 << 5 | 1 << 13
    # Combine enable bits with mode bits.
    slli  a3, a3, 11
    or    t0, t0, a3
    csrw  mstatus, t0
    csrw  mepc, a1
    csrw  satp, a2
    # li    t1, 0xaaa
    # csrw  mie, t1
    # la    t2, m_trap_vector
    # csrw  mtvec, t2
    # This fence forces the MMU to flush the TLB. However, since
    # we're using the PID as the address space identifier, we might
    # only need this when we create a process. Right now, this ensures
    # correctness, however it isn't the most efficient.
    # sfence.vma
    # A0 is the context frame, so we need to reload it back
    # and mret so we can start running the program.
    mv  t6, a0
    .set  i, 0
    .rept 32
        load_fp %i
        .set i, i+1
    .endr
    1:
    .set  i, 1
    .rept 31
        load_gp %i, t6
        .set  i, i+1
    .endr

    mret
