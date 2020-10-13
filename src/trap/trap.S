.section .text.tongos.trap

.weak trap_handler
.balign 4, 0
.global trap_handler

.weak machine_software_interrupt
.balign 4, 0
.global machine_software_interrupt

.option norvc
.weak mtvec_clint_vector_table
.balign 256, 0
.global mtvec_clint_vector_table
mtvec_clint_vector_table:

mcause_code_0:
  j trap_handler
mcause_code_1:
  j trap_handler
mcause_code_2:
  j trap_handler
mcause_code_3:
  j machine_software_interrupt
mcause_code_4:
  j trap_handler
mcause_code_5:
  j trap_handler
mcause_code_6:
  j trap_handler
mcause_code_7:
  j trap_handler
mcause_code_8:
  j trap_handler
mcause_code_9:
  j trap_handler
mcause_code_10:
  j trap_handler
mcause_code_11:
  j trap_handler
mcause_code_12:
  j trap_handler
mcause_code_13:
  j trap_handler
mcause_code_14:
  j trap_handler
mcause_code_15:
  j trap_handler
