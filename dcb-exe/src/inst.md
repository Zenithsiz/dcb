# Cpu instructions

This module defines all instructions for the psx cpu, the
`MIPS R3051`, following [Nocash's specifications](https://problemkaputt.de/psx-spx.htm).

The instructions are split across 3 main types,
- `[basic::Inst]`, which defines all 'basic' instructions, i.e. all instructions which are
  a single word in size.
- `[pseudo::Inst]`, instructions which are decoded from basic instructions and that represent either
  a simplified version of an instruction, or multiple instructions (such as `la $a0, 0x80001000` == `lui $a0, 0x8000 / addiu $ao, 0x1000`).
- `[Directive]`, which represent data, rather than instructions, such as `dw` and `.ascii`.

See each instruction's module for information on how they are decoded and their variants.

# Instructions

## Basic

### ALU

| Mnemonic | Syntax                  | Meaning              | Description                        |
| -------- | ----------------------- | -------------------- | ---------------------------------- |
| `addi`   | `addi  $dst, $lhs, val` | `$dst = $lhs + val`  | `val` is `i16`. Checks on overflow |
| ^^       | `addi  $dst, val`       | `$dst += val`        | ^^                                 |
| `addiu`  | `addiu $dst, $lhs, val` | `$dst = $lhs + val`  | `val` is `i16`. No overflow        |
| ^^       | `addiu $dst, val`       | `$dst += val`        | ^^                                 |
| `slti`   | `slti  $dst, $lhs, val` | `$dst = $lhs < val`  | `val` is `i16`. Result is 0 or 1   |
| `sltiu`  | `sltiu $dst, $lhs, val` | `$dst = $lhs < val`  | `val` is `u16`. Result is 0 or 1   |
| `andi`   | `andi  $dst, $lhs, val` | `$dst = $lhs & val`  | `val` is `u16`.                    |
| ^^       | `andi  $dst, val`       | `$dst &= val`        | ^^                                 |
| `ori`    | `ori   $dst, $lhs, val` | `$dst = $lhs \| val` | `val` is `u16`.                    |
| ^^       | `ori   $dst, val`       | `$dst \|= val`       | ^^                                 |
| `xori`   | `xori  $dst, $lhs, val` | `$dst = $lhs ^ val`  | `val` is `u16`.                    |
| ^^       | `xori  $dst, val`       | `$dst ^= val`        | ^^                                 |
