.syntax unified
.arch armv7-a
.fpu neon-vfpv4
.arm
.section .text.context, "ax"
.align 2
.global switch_to_thread
.type switch_to_thread, %function
switch_to_thread:
    push {r4-r11, r12, lr}
    vpush {d8-d15}
    vmrs r2, fpscr
    push {r2, r3}
    str sp, [r0]
    mov sp, r1
    pop {r2, r3}
    vmsr fpscr, r2
    vpop {d8-d15}
    pop {r4-r11, r12, lr}
    bx lr
.size switch_to_thread, . - switch_to_thread

.section .text.vectors, "ax"
.align 5
.global exception_vectors
exception_vectors:
    b exception_reset
    b exception_undefined
    b exception_svc
    b exception_prefetch
    b exception_data
    b exception_reserved
    b irq_entry
    b exception_fiq

irq_entry:
    sub lr, lr, #4
    srsdb sp!, #0x13
    cpsid if, #0x13
    push {r0-r12, lr}
    mov r4, sp
    bic sp, sp, #7
    vpush {d0-d15}
    vpush {d16-d31}
    vmrs r0, fpscr
    push {r0, r4}
    bl irq_handler
    pop {r0, r4}
    vmsr fpscr, r0
    vpop {d16-d31}
    vpop {d0-d15}
    mov sp, r4
    pop {r0-r12, lr}
    rfeia sp!

exception_reset:
    mov r0, #0
    b fatal_exception
exception_undefined:
    mov r0, #1
    sub lr, lr, #4
    b fatal_exception
exception_svc:
    mov r0, #2
    sub lr, lr, #4
    b fatal_exception
exception_prefetch:
    mov r0, #3
    sub lr, lr, #4
    mrc p15, #0, r2, c5, c0, #1
    mrc p15, #0, r3, c6, c0, #2
    b fatal_abort
exception_data:
    mov r0, #4
    sub lr, lr, #8
    mrc p15, #0, r2, c5, c0, #0
    mrc p15, #0, r3, c6, c0, #0
    b fatal_abort
exception_reserved:
    mov r0, #5
    b fatal_exception
exception_fiq:
    mov r0, #7
    sub lr, lr, #4
fatal_exception:
    mrs r2, spsr
    mov r3, #0
fatal_abort:
    mov r1, lr
    cpsid if
    bic sp, sp, #7
    bl exception_handler
1:
    wfi
    b 1b
.section .note.GNU-stack, "", %progbits
