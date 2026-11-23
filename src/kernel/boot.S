.syntax unified
.cpu cortex-a7
.fpu neon-vfpv4
.arm
.section .text.boot, "ax"
.global _start
.type _start, %function
_start:
    cpsid if
    mrc p15, #0, r3, c0, c0, #5
    and r3, r3, #3
    cmp r3, #0
    bne halt
    mov r4, r0
    mov r5, r1
    mov r6, r2
    mrs r3, cpsr
    and r3, r3, #0x1f
    cmp r3, #0x1a
    bne supervisor
    adr r3, supervisor
    msr ELR_hyp, r3
    mov r3, #0xd3
    msr SPSR_hyp, r3
    eret
supervisor:
    cps #0x13
    mrc p15, #0, r3, c1, c0, #0
    bic r3, r3, #1
    bic r3, r3, #(1 << 2)
    bic r3, r3, #(1 << 12)
    bic r3, r3, #(1 << 13)
    dsb sy
    mcr p15, #0, r3, c1, c0, #0
    isb
    mrc p15, #0, r3, c1, c0, #2
    orr r3, r3, #(0xf << 20)
    mcr p15, #0, r3, c1, c0, #2
    isb
    mov r3, #(1 << 30)
    vmsr fpexc, r3
    mov r3, #0
    vmsr fpscr, r3
    ldr r0, =__bss_start
    ldr r1, =__bss_end
1:
    cmp r0, r1
    strlo r3, [r0], #4
    blo 1b
    cps #0x12
    ldr sp, =irq_stack_top
    cps #0x17
    ldr sp, =abort_stack_top
    cps #0x1b
    ldr sp, =undefined_stack_top
    cps #0x11
    ldr sp, =fiq_stack_top
    cps #0x13
    ldr sp, =kernel_stack_top
    ldr r3, =exception_vectors
    mcr p15, #0, r3, c12, c0, #0
    dsb sy
    isb
    mov r0, r4
    mov r1, r5
    mov r2, r6
    bl kernel_main
halt:
    wfi
    b halt
.size _start, . - _start

.section .bss.stacks, "aw", %nobits
.balign 16
kernel_stack:
    .space 16384
kernel_stack_top:
irq_stack:
    .space 4096
irq_stack_top:
abort_stack:
    .space 4096
abort_stack_top:
undefined_stack:
    .space 4096
undefined_stack_top:
fiq_stack:
    .space 4096
fiq_stack_top:
.section .note.GNU-stack, "", %progbits
