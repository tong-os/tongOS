# trap.S
# Trap handler and global context
# Steve Operating System
# Stephen Marz
# tongOs team

.section .text.tongos.trap
.option norvc

.global __tong_os_trap_machine_mode
.align 4
__tong_os_trap_machine_mode:
    addi sp, sp, -68*8
    # Save general purpose registers
    sd zero, 0*8(sp)
    sd ra, 1*8(sp)
    sd sp, 2*8(sp)
    sd gp, 3*8(sp)
    sd tp, 4*8(sp)
    sd t0, 5*8(sp)
    sd t1, 6*8(sp)
    sd t2, 7*8(sp)
    sd s0, 8*8(sp)
    sd s1, 9*8(sp)
    sd a0, 10*8(sp)
    sd a1, 11*8(sp)
    sd a2, 12*8(sp)
    sd a3, 13*8(sp)
    sd a4, 14*8(sp)
    sd a5, 15*8(sp)
    sd a6, 16*8(sp)
    sd a7, 17*8(sp)
    sd s2, 18*8(sp)
    sd s3, 19*8(sp)
    sd s4, 20*8(sp)
    sd s5, 21*8(sp)
    sd s6, 22*8(sp)
    sd s7, 23*8(sp)
    sd s8, 24*8(sp)
    sd s9, 25*8(sp)
    sd s10, 26*8(sp)
    sd s11, 27*8(sp)
    sd t3, 28*8(sp)
    sd t4, 29*8(sp)
    sd t5, 30*8(sp)
    sd t6, 31*8(sp)

    # Save SATP
    csrrw t1, satp, zero
    # 64 = 32 gp + 32 fp + satp - 1
    sd t1, 64*8(sp)

    # Save PC
    csrrw t2, mepc, zero
    # 65 = 32 gp + 32 fp + satp + pc - 1
    sd t2, 65*8(sp)

    # Get mstatus
    csrr t3, mstatus
    csrw mstatus, t3

    # Save Global_interrupt_enable
    li t4, 1 << 7
    and t4, t3, t4
    sd t4, 66*8(sp)

    # Save machine previous protection
    li t5, 11 << 11
    and t5, t3, t5
    sd t5, 67*8(sp)

    # Prepare arg 0 as trap_frame
    mv a0, sp

    # load kernel stack
    la sp, _stack_end

    csrr t2, mhartid
    li t0, 0
    la t1, _stack_size
_set_hart_stack_pointer:
    beq t0, t2, _call_mtrap
    sub sp, sp, t1
    addi t0, t0, 1
    j _set_hart_stack_pointer

_call_mtrap:
    call tong_os_trap

.global __tong_os_switch_to_process
__tong_os_switch_to_process:
    # a0 = process trap frame on stack end

    # Load program counter
    ld a1, 65*8(a0)
    # Load satp
    ld a2, 64*8(a0)
    # Load processor mode
    ld a3, 67*8(a0)
    # Load global interrupt enable
    ld a4, 66*8(a0)

    # shift mode to correct place
    slli  a3, a3, 11
    # merge flags and mode
    or    t0, a4, a3
    # write to mstatus
    csrw  mstatus, t0

    # write mepc with process pc
    csrw  mepc, a1

    # write satp with process satp
    csrw  satp, a2

    # load_context
    mv t6, a0

    # Save general purpose registers
    ld zero, 0*8(t6)
    ld ra, 1*8(t6)
    ld sp, 2*8(t6)
    ld gp, 3*8(t6)
    ld tp, 4*8(t6)
    ld t0, 5*8(t6)
    ld t1, 6*8(t6)
    ld t2, 7*8(t6)
    ld s0, 8*8(t6)
    ld s1, 9*8(t6)
    ld a0, 10*8(t6)
    ld a1, 11*8(t6)
    ld a2, 12*8(t6)
    ld a3, 13*8(t6)
    ld a4, 14*8(t6)
    ld a5, 15*8(t6)
    ld a6, 16*8(t6)
    ld a7, 17*8(t6)
    ld s2, 18*8(t6)
    ld s3, 19*8(t6)
    ld s4, 20*8(t6)
    ld s5, 21*8(t6)
    ld s6, 22*8(t6)
    ld s7, 23*8(t6)
    ld s8, 24*8(t6)
    ld s9, 25*8(t6)
    ld s10, 26*8(t6)
    ld s11, 27*8(t6)
    ld t3, 28*8(t6)
    ld t4, 29*8(t6)
    ld t5, 30*8(t6)
    ld t6, 31*8(t6)

    # restore stack
    add sp, sp, 68*8

    mret

