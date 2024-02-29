.cpu cortex-m0
.thumb
.equ GPIO_OE, 0x20
.equ GPIO_OE_SET, 0x24
.equ GPIO_OE_CLR, 0x28


/**
 * Connects selected GPIO pin to selected peripherial by using GPIO CTRL register.
 * r0 - GPIO Pin
 * r1 - GPIO Function (cf. RP2040 Datasheet 1.4.3)
 * */
.thumb_func
.global GPIO_function_select
.align 4
GPIO_function_select:
    ldr  r3, IO_BANK0_BASE

    movs r2, #8
    muls r2, r2, r0                 @ calculate offset for GPIO_N_CTRL (minus 0x04)
    adds r2 , #0x04                  @ GPIO0_CTRL offset
    add r3, r3, r2                 @ add calculated offset

    str r1 , [r3]                   @ write specfied function

    bx lr


@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@@
@@          PIN INPUTS
@@
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

/**
 * Configs specified pin to work as input.
 * r0 - GPIO Pin
 * r1 - pull mask (up/down) -> down: 0, up: true
 * */
.thumb_func
.global init_pin_input_with_pull
.align 4
init_pin_input_with_pull:
    push {lr}
    ldr r2, =in_pin
    str r0, [r2]
    ldr r2, =pull_mask
    str r1, [r2]

    ldr r0, in_pin
    movs r1, #5
    bl  GPIO_function_select

    ldr r0, in_pin
    ldr r1, pull_mask

    ldr r3, PADS_BANK0_BASE
    movs r2, #0x04
    muls r0, r0, r2                 @ calculate pin offset
    adds r0, r0, #0x04              @ add offset start (GPIO0)
    add r0, r0, r3

    @ OUTPUT DISABLE + INPUT ENABLE = 0xC0
    @ PULL DOWN - BIT 2 ; PULL UP - BIT 3
    cmp r1, #0
    bne .pull_up
    .pull_down:
        movs r1, #0xC4
        b .store_config
    .pull_up:
        movs r1, #0xC8
    .store_config:
        str  r1, [r0]

    pop  {pc}

.align 4
in_pin:             .word 0
PADS_BANK0_BASE:    .word 0x4001c000
SIO_BASE:           .word 0xd0000000
