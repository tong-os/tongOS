.section .text.tongos.init

.global _tongos_entry
_tongos_entry:
    .cfi_startproc
    .cfi_undefined ra

set_global_pointer:
.option push
.option norelax
    la gp, _global_pointer
.option pop

set_early_trap_vector:
    csrw mie, zero
    la t0, early_trap_vector
    csrw mtvec, t0

set_stack_pointer:
    la sp, _stack_end

partition_stack_between_harts:
    csrr a0, mhartid
    li t0, 0
    la t1, _stack_size
set_hart_stack_pointer:
    beq t0, a0, call_kinit
    add sp, sp, t1
    addi t0, t0, 1
    j set_hart_stack_pointer

call_kinit:
    csrr a0, mhartid
    li a1, 0
    li a2, 0
    call kinit

return_from_kinit_is_error:
    /* If that handler returns then there's not a whole lot we can do.  Just
     * try to make some noise. */
    la t0, 1f
    csrw mtvec, t0
1:
    lw t1, 0(x0)
    j 1b

    /* Allows stack_trace */
    .cfi_endproc

.global early_trap_vector
.align 4
early_trap_vector:
    .cfi_startproc
    csrr t0, mcause
    csrr t1, mepc
    csrr t2, mtval
    j early_trap_vector
    .cfi_endproc
