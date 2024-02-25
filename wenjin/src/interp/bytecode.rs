use sti::alloc::Alloc;
use sti::vec::Vec;
use sti::static_assert;


// IMPORTANT: keep `Op::num_extra_words` in sync.

#[allow(non_camel_case_types)]
#[repr(align(4))]
#[derive(Clone, Copy, Debug)]
pub enum Op {
    UNIMPLEMENTED,
    UNREACHABLE,
    JUMP            { delta: i16 },
    JUMP_FALSE      { src: u8, delta: i16 },
    JUMP_TRUE       { src: u8, delta: i16 },
    JUMP_TABLE      { src: u8, len: u8 },
    RETURN          { base: u8, num_rets: u8 },
    CALL_INDIRECT   { base: u8 },
    CALL_BYTECODE   { base: u8 },
    CALL_TABLE      { base: u8, src: u8 },
    SELECT          { dst: u8, src1: u8, src2: u8 },
    I32_CONST       { dst: u8 },
    I64_CONST       { dst: u8 },
    F32_CONST       { dst: u8 },
    F64_CONST       { dst: u8 },
    COPY            { dst: u8, src: u8 },
    GLOBAL_GET      { dst: u8 },
    GLOBAL_SET      { src: u8 },
    LOAD_I32        { dst: u8, addr: u8 },
    LOAD_I32_8S     { dst: u8, addr: u8 },
    LOAD_I32_8U     { dst: u8, addr: u8 },
    LOAD_I32_16S    { dst: u8, addr: u8 },
    LOAD_I32_16U    { dst: u8, addr: u8 },
    LOAD_I64        { dst: u8, addr: u8 },
    LOAD_I64_8S     { dst: u8, addr: u8 },
    LOAD_I64_8U     { dst: u8, addr: u8 },
    LOAD_I64_16S    { dst: u8, addr: u8 },
    LOAD_I64_16U    { dst: u8, addr: u8 },
    LOAD_I64_32S    { dst: u8, addr: u8 },
    LOAD_I64_32U    { dst: u8, addr: u8 },
    LOAD_F32        { dst: u8, addr: u8 },
    LOAD_F64        { dst: u8, addr: u8 },
    STORE_I32       { addr: u8, src: u8 },
    STORE_I32_8     { addr: u8, src: u8 },
    STORE_I32_16    { addr: u8, src: u8 },
    STORE_I64       { addr: u8, src: u8 },
    STORE_I64_8     { addr: u8, src: u8 },
    STORE_I64_16    { addr: u8, src: u8 },
    STORE_I64_32    { addr: u8, src: u8 },
    STORE_F32       { addr: u8, src: u8 },
    STORE_F64       { addr: u8, src: u8 },
    I32_EQZ             { dst: u8, src: u8 },
    I64_EQZ             { dst: u8, src: u8 },
    I32_CLZ             { dst: u8, src: u8 },
    I32_CTZ             { dst: u8, src: u8 },
    I32_POPCNT          { dst: u8, src: u8 },
    I32_EXTEND8_S       { dst: u8, src: u8 },
    I32_EXTEND16_S      { dst: u8, src: u8 },
    I64_CLZ             { dst: u8, src: u8 },
    I64_CTZ             { dst: u8, src: u8 },
    I64_POPCNT          { dst: u8, src: u8 },
    I64_EXTEND8_S       { dst: u8, src: u8 },
    I64_EXTEND16_S      { dst: u8, src: u8 },
    I64_EXTEND32_S      { dst: u8, src: u8 },
    F32_ABS             { dst: u8, src: u8 },
    F32_NEG             { dst: u8, src: u8 },
    F32_CEIL            { dst: u8, src: u8 },
    F32_FLOOR           { dst: u8, src: u8 },
    F32_TRUNC           { dst: u8, src: u8 },
    F32_NEAREST         { dst: u8, src: u8 },
    F32_SQRT            { dst: u8, src: u8 },
    F64_ABS             { dst: u8, src: u8 },
    F64_NEG             { dst: u8, src: u8 },
    F64_CEIL            { dst: u8, src: u8 },
    F64_FLOOR           { dst: u8, src: u8 },
    F64_TRUNC           { dst: u8, src: u8 },
    F64_NEAREST         { dst: u8, src: u8 },
    F64_SQRT            { dst: u8, src: u8 },
    I32_WRAP_I64        { dst: u8, src: u8 },
    I64_EXTEND_I32_S    { dst: u8, src: u8 },
    I64_EXTEND_I32_U    { dst: u8, src: u8 },
    I32_TRUNC_F32_S     { dst: u8, src: u8 },
    I32_TRUNC_F32_U     { dst: u8, src: u8 },
    F32_CONVERT_I32_S   { dst: u8, src: u8 },
    F32_CONVERT_I32_U   { dst: u8, src: u8 },
    I32_TRUNC_F64_S     { dst: u8, src: u8 },
    I32_TRUNC_F64_U     { dst: u8, src: u8 },
    F64_CONVERT_I32_S   { dst: u8, src: u8 },
    F64_CONVERT_I32_U   { dst: u8, src: u8 },
    I64_TRUNC_F32_S     { dst: u8, src: u8 },
    I64_TRUNC_F32_U     { dst: u8, src: u8 },
    F32_CONVERT_I64_S   { dst: u8, src: u8 },
    F32_CONVERT_I64_U   { dst: u8, src: u8 },
    I64_TRUNC_F64_S     { dst: u8, src: u8 },
    I64_TRUNC_F64_U     { dst: u8, src: u8 },
    F64_CONVERT_I64_S   { dst: u8, src: u8 },
    F64_CONVERT_I64_U   { dst: u8, src: u8 },
    F32_DEMOTE_F64      { dst: u8, src: u8 },
    F64_PROMOTE_F32     { dst: u8, src: u8 },
    I32_EQ          { dst: u8, src1: u8, src2: u8 },
    I32_NE          { dst: u8, src1: u8, src2: u8 },
    I32_LE_S        { dst: u8, src1: u8, src2: u8 },
    I32_LE_U        { dst: u8, src1: u8, src2: u8 },
    I32_LT_S        { dst: u8, src1: u8, src2: u8 },
    I32_LT_U        { dst: u8, src1: u8, src2: u8 },
    I32_GE_S        { dst: u8, src1: u8, src2: u8 },
    I32_GE_U        { dst: u8, src1: u8, src2: u8 },
    I32_GT_S        { dst: u8, src1: u8, src2: u8 },
    I32_GT_U        { dst: u8, src1: u8, src2: u8 },
    I64_EQ          { dst: u8, src1: u8, src2: u8 },
    I64_NE          { dst: u8, src1: u8, src2: u8 },
    I64_LE_S        { dst: u8, src1: u8, src2: u8 },
    I64_LE_U        { dst: u8, src1: u8, src2: u8 },
    I64_LT_S        { dst: u8, src1: u8, src2: u8 },
    I64_LT_U        { dst: u8, src1: u8, src2: u8 },
    I64_GE_S        { dst: u8, src1: u8, src2: u8 },
    I64_GE_U        { dst: u8, src1: u8, src2: u8 },
    I64_GT_S        { dst: u8, src1: u8, src2: u8 },
    I64_GT_U        { dst: u8, src1: u8, src2: u8 },
    F32_EQ          { dst: u8, src1: u8, src2: u8 },
    F32_NE          { dst: u8, src1: u8, src2: u8 },
    F32_LE          { dst: u8, src1: u8, src2: u8 },
    F32_LT          { dst: u8, src1: u8, src2: u8 },
    F32_GE          { dst: u8, src1: u8, src2: u8 },
    F32_GT          { dst: u8, src1: u8, src2: u8 },
    F64_EQ          { dst: u8, src1: u8, src2: u8 },
    F64_NE          { dst: u8, src1: u8, src2: u8 },
    F64_LE          { dst: u8, src1: u8, src2: u8 },
    F64_LT          { dst: u8, src1: u8, src2: u8 },
    F64_GE          { dst: u8, src1: u8, src2: u8 },
    F64_GT          { dst: u8, src1: u8, src2: u8 },
    I32_ADD         { dst: u8, src1: u8, src2: u8 },
    I32_SUB         { dst: u8, src1: u8, src2: u8 },
    I32_MUL         { dst: u8, src1: u8, src2: u8 },
    I32_DIV_S       { dst: u8, src1: u8, src2: u8 },
    I32_DIV_U       { dst: u8, src1: u8, src2: u8 },
    I32_REM_S       { dst: u8, src1: u8, src2: u8 },
    I32_REM_U       { dst: u8, src1: u8, src2: u8 },
    I32_AND         { dst: u8, src1: u8, src2: u8 },
    I32_OR          { dst: u8, src1: u8, src2: u8 },
    I32_XOR         { dst: u8, src1: u8, src2: u8 },
    I32_SHL         { dst: u8, src1: u8, src2: u8 },
    I32_SHR_S       { dst: u8, src1: u8, src2: u8 },
    I32_SHR_U       { dst: u8, src1: u8, src2: u8 },
    I32_ROTL        { dst: u8, src1: u8, src2: u8 },
    I32_ROTR        { dst: u8, src1: u8, src2: u8 },
    I64_ADD         { dst: u8, src1: u8, src2: u8 },
    I64_SUB         { dst: u8, src1: u8, src2: u8 },
    I64_MUL         { dst: u8, src1: u8, src2: u8 },
    I64_DIV_S       { dst: u8, src1: u8, src2: u8 },
    I64_DIV_U       { dst: u8, src1: u8, src2: u8 },
    I64_REM_S       { dst: u8, src1: u8, src2: u8 },
    I64_REM_U       { dst: u8, src1: u8, src2: u8 },
    I64_AND         { dst: u8, src1: u8, src2: u8 },
    I64_OR          { dst: u8, src1: u8, src2: u8 },
    I64_XOR         { dst: u8, src1: u8, src2: u8 },
    I64_SHL         { dst: u8, src1: u8, src2: u8 },
    I64_SHR_S       { dst: u8, src1: u8, src2: u8 },
    I64_SHR_U       { dst: u8, src1: u8, src2: u8 },
    I64_ROTL        { dst: u8, src1: u8, src2: u8 },
    I64_ROTR        { dst: u8, src1: u8, src2: u8 },
    F32_ADD         { dst: u8, src1: u8, src2: u8 },
    F32_SUB         { dst: u8, src1: u8, src2: u8 },
    F32_MUL         { dst: u8, src1: u8, src2: u8 },
    F32_DIV         { dst: u8, src1: u8, src2: u8 },
    F32_MIN         { dst: u8, src1: u8, src2: u8 },
    F32_MAX         { dst: u8, src1: u8, src2: u8 },
    F32_COPYSIGN    { dst: u8, src1: u8, src2: u8 },
    F64_ADD         { dst: u8, src1: u8, src2: u8 },
    F64_SUB         { dst: u8, src1: u8, src2: u8 },
    F64_MUL         { dst: u8, src1: u8, src2: u8 },
    F64_DIV         { dst: u8, src1: u8, src2: u8 },
    F64_MIN         { dst: u8, src1: u8, src2: u8 },
    F64_MAX         { dst: u8, src1: u8, src2: u8 },
    F64_COPYSIGN    { dst: u8, src1: u8, src2: u8 },
    MEMORY_SIZE     { dst: u8 },
    MEMORY_GROW     { dst: u8, delta: u8 },
    MEMORY_COPY     { dst_addr: u8, src_addr: u8, len: u8 },
    MEMORY_FILL     { dst_addr: u8, val: u8, len: u8 },
}

static_assert!(core::mem::size_of::<Op>() == 4);


use crate::wasm::operand::{Load, Store};

use crate::interp::instr::{Op1, Op2};

impl Op {
    pub fn from_load(load: Load, dst: u8, addr: u8) -> Op {
        match load {
            Load::I32     => Op::LOAD_I32     { dst, addr },
            Load::I32_8S  => Op::LOAD_I32_8S  { dst, addr },
            Load::I32_8U  => Op::LOAD_I32_8U  { dst, addr },
            Load::I32_16S => Op::LOAD_I32_16S { dst, addr },
            Load::I32_16U => Op::LOAD_I32_16U { dst, addr },
            Load::I64     => Op::LOAD_I64     { dst, addr },
            Load::I64_8S  => Op::LOAD_I64_8S  { dst, addr },
            Load::I64_8U  => Op::LOAD_I64_8U  { dst, addr },
            Load::I64_16S => Op::LOAD_I64_16S { dst, addr },
            Load::I64_16U => Op::LOAD_I64_16U { dst, addr },
            Load::I64_32S => Op::LOAD_I64_32S { dst, addr },
            Load::I64_32U => Op::LOAD_I64_32U { dst, addr },
            Load::F32     => Op::LOAD_F32     { dst, addr },
            Load::F64     => Op::LOAD_F64     { dst, addr },
        }
    }

    pub fn from_store(store: Store, addr: u8, src: u8) -> Op {
        match store {
            Store::I32    => Op::STORE_I32    { addr, src },
            Store::I32_8  => Op::STORE_I32_8  { addr, src },
            Store::I32_16 => Op::STORE_I32_16 { addr, src },
            Store::I64    => Op::STORE_I64    { addr, src },
            Store::I64_8  => Op::STORE_I64_8  { addr, src },
            Store::I64_16 => Op::STORE_I64_16 { addr, src },
            Store::I64_32 => Op::STORE_I64_32 { addr, src },
            Store::F32    => Op::STORE_F32    { addr, src },
            Store::F64    => Op::STORE_F64    { addr, src },
        }
    }


    pub fn from_op1(op: Op1, dst: u8, src: u8) -> Op {
        match op {
            Op1::I32_EQZ           => Op::I32_EQZ           { dst, src },
            Op1::I64_EQZ           => Op::I64_EQZ           { dst, src },
            Op1::I32_CLZ           => Op::I32_CLZ           { dst, src },
            Op1::I32_CTZ           => Op::I32_CTZ           { dst, src },
            Op1::I32_POPCNT        => Op::I32_POPCNT        { dst, src },
            Op1::I32_EXTEND8_S     => Op::I32_EXTEND8_S     { dst, src },
            Op1::I32_EXTEND16_S    => Op::I32_EXTEND16_S    { dst, src },
            Op1::I64_CLZ           => Op::I64_CLZ           { dst, src },
            Op1::I64_CTZ           => Op::I64_CTZ           { dst, src },
            Op1::I64_POPCNT        => Op::I64_POPCNT        { dst, src },
            Op1::I64_EXTEND8_S     => Op::I64_EXTEND8_S     { dst, src },
            Op1::I64_EXTEND16_S    => Op::I64_EXTEND16_S    { dst, src },
            Op1::I64_EXTEND32_S    => Op::I64_EXTEND32_S    { dst, src },
            Op1::F32_ABS           => Op::F32_ABS           { dst, src },
            Op1::F32_NEG           => Op::F32_NEG           { dst, src },
            Op1::F32_CEIL          => Op::F32_CEIL          { dst, src },
            Op1::F32_FLOOR         => Op::F32_FLOOR         { dst, src },
            Op1::F32_TRUNC         => Op::F32_TRUNC         { dst, src },
            Op1::F32_NEAREST       => Op::F32_NEAREST       { dst, src },
            Op1::F32_SQRT          => Op::F32_SQRT          { dst, src },
            Op1::F64_ABS           => Op::F64_ABS           { dst, src },
            Op1::F64_NEG           => Op::F64_NEG           { dst, src },
            Op1::F64_CEIL          => Op::F64_CEIL          { dst, src },
            Op1::F64_FLOOR         => Op::F64_FLOOR         { dst, src },
            Op1::F64_TRUNC         => Op::F64_TRUNC         { dst, src },
            Op1::F64_NEAREST       => Op::F64_NEAREST       { dst, src },
            Op1::F64_SQRT          => Op::F64_SQRT          { dst, src },
            Op1::I32_WRAP_I64      => Op::I32_WRAP_I64      { dst, src },
            Op1::I64_EXTEND_I32_S  => Op::I64_EXTEND_I32_S  { dst, src },
            Op1::I64_EXTEND_I32_U  => Op::I64_EXTEND_I32_U  { dst, src },
            Op1::I32_TRUNC_F32_S   => Op::I32_TRUNC_F32_S   { dst, src },
            Op1::I32_TRUNC_F32_U   => Op::I32_TRUNC_F32_U   { dst, src },
            Op1::F32_CONVERT_I32_S => Op::F32_CONVERT_I32_S { dst, src },
            Op1::F32_CONVERT_I32_U => Op::F32_CONVERT_I32_U { dst, src },
            Op1::I32_TRUNC_F64_S   => Op::I32_TRUNC_F64_S   { dst, src },
            Op1::I32_TRUNC_F64_U   => Op::I32_TRUNC_F64_U   { dst, src },
            Op1::F64_CONVERT_I32_S => Op::F64_CONVERT_I32_S { dst, src },
            Op1::F64_CONVERT_I32_U => Op::F64_CONVERT_I32_U { dst, src },
            Op1::I64_TRUNC_F32_S   => Op::I64_TRUNC_F32_S   { dst, src },
            Op1::I64_TRUNC_F32_U   => Op::I64_TRUNC_F32_U   { dst, src },
            Op1::F32_CONVERT_I64_S => Op::F32_CONVERT_I64_S { dst, src },
            Op1::F32_CONVERT_I64_U => Op::F32_CONVERT_I64_U { dst, src },
            Op1::I64_TRUNC_F64_S   => Op::I64_TRUNC_F64_S   { dst, src },
            Op1::I64_TRUNC_F64_U   => Op::I64_TRUNC_F64_U   { dst, src },
            Op1::F64_CONVERT_I64_S => Op::F64_CONVERT_I64_S { dst, src },
            Op1::F64_CONVERT_I64_U => Op::F64_CONVERT_I64_U { dst, src },
            Op1::F32_DEMOTE_F64    => Op::F32_DEMOTE_F64    { dst, src },
            Op1::F64_PROMOTE_F32   => Op::F64_PROMOTE_F32   { dst, src },
        }
    }

    pub fn from_op2(op: Op2, dst: u8, src1: u8, src2: u8) -> Op {
        match op {
            Op2::I32_EQ       => Op::I32_EQ       { dst, src1, src2 },
            Op2::I32_NE       => Op::I32_NE       { dst, src1, src2 },
            Op2::I32_LE_S     => Op::I32_LE_S     { dst, src1, src2 },
            Op2::I32_LE_U     => Op::I32_LE_U     { dst, src1, src2 },
            Op2::I32_LT_S     => Op::I32_LT_S     { dst, src1, src2 },
            Op2::I32_LT_U     => Op::I32_LT_U     { dst, src1, src2 },
            Op2::I32_GE_S     => Op::I32_GE_S     { dst, src1, src2 },
            Op2::I32_GE_U     => Op::I32_GE_U     { dst, src1, src2 },
            Op2::I32_GT_S     => Op::I32_GT_S     { dst, src1, src2 },
            Op2::I32_GT_U     => Op::I32_GT_U     { dst, src1, src2 },
            Op2::I64_EQ       => Op::I64_EQ       { dst, src1, src2 },
            Op2::I64_NE       => Op::I64_NE       { dst, src1, src2 },
            Op2::I64_LE_S     => Op::I64_LE_S     { dst, src1, src2 },
            Op2::I64_LE_U     => Op::I64_LE_U     { dst, src1, src2 },
            Op2::I64_LT_S     => Op::I64_LT_S     { dst, src1, src2 },
            Op2::I64_LT_U     => Op::I64_LT_U     { dst, src1, src2 },
            Op2::I64_GE_S     => Op::I64_GE_S     { dst, src1, src2 },
            Op2::I64_GE_U     => Op::I64_GE_U     { dst, src1, src2 },
            Op2::I64_GT_S     => Op::I64_GT_S     { dst, src1, src2 },
            Op2::I64_GT_U     => Op::I64_GT_U     { dst, src1, src2 },
            Op2::F32_EQ       => Op::F32_EQ       { dst, src1, src2 },
            Op2::F32_NE       => Op::F32_NE       { dst, src1, src2 },
            Op2::F32_LE       => Op::F32_LE       { dst, src1, src2 },
            Op2::F32_LT       => Op::F32_LT       { dst, src1, src2 },
            Op2::F32_GE       => Op::F32_GE       { dst, src1, src2 },
            Op2::F32_GT       => Op::F32_GT       { dst, src1, src2 },
            Op2::F64_EQ       => Op::F64_EQ       { dst, src1, src2 },
            Op2::F64_NE       => Op::F64_NE       { dst, src1, src2 },
            Op2::F64_LE       => Op::F64_LE       { dst, src1, src2 },
            Op2::F64_LT       => Op::F64_LT       { dst, src1, src2 },
            Op2::F64_GE       => Op::F64_GE       { dst, src1, src2 },
            Op2::F64_GT       => Op::F64_GT       { dst, src1, src2 },
            Op2::I32_ADD      => Op::I32_ADD      { dst, src1, src2 },
            Op2::I32_SUB      => Op::I32_SUB      { dst, src1, src2 },
            Op2::I32_MUL      => Op::I32_MUL      { dst, src1, src2 },
            Op2::I32_DIV_S    => Op::I32_DIV_S    { dst, src1, src2 },
            Op2::I32_DIV_U    => Op::I32_DIV_U    { dst, src1, src2 },
            Op2::I32_REM_S    => Op::I32_REM_S    { dst, src1, src2 },
            Op2::I32_REM_U    => Op::I32_REM_U    { dst, src1, src2 },
            Op2::I32_AND      => Op::I32_AND      { dst, src1, src2 },
            Op2::I32_OR       => Op::I32_OR       { dst, src1, src2 },
            Op2::I32_XOR      => Op::I32_XOR      { dst, src1, src2 },
            Op2::I32_SHL      => Op::I32_SHL      { dst, src1, src2 },
            Op2::I32_SHR_S    => Op::I32_SHR_S    { dst, src1, src2 },
            Op2::I32_SHR_U    => Op::I32_SHR_U    { dst, src1, src2 },
            Op2::I32_ROTL     => Op::I32_ROTL     { dst, src1, src2 },
            Op2::I32_ROTR     => Op::I32_ROTR     { dst, src1, src2 },
            Op2::I64_ADD      => Op::I64_ADD      { dst, src1, src2 },
            Op2::I64_SUB      => Op::I64_SUB      { dst, src1, src2 },
            Op2::I64_MUL      => Op::I64_MUL      { dst, src1, src2 },
            Op2::I64_DIV_S    => Op::I64_DIV_S    { dst, src1, src2 },
            Op2::I64_DIV_U    => Op::I64_DIV_U    { dst, src1, src2 },
            Op2::I64_REM_S    => Op::I64_REM_S    { dst, src1, src2 },
            Op2::I64_REM_U    => Op::I64_REM_U    { dst, src1, src2 },
            Op2::I64_AND      => Op::I64_AND      { dst, src1, src2 },
            Op2::I64_OR       => Op::I64_OR       { dst, src1, src2 },
            Op2::I64_XOR      => Op::I64_XOR      { dst, src1, src2 },
            Op2::I64_SHL      => Op::I64_SHL      { dst, src1, src2 },
            Op2::I64_SHR_S    => Op::I64_SHR_S    { dst, src1, src2 },
            Op2::I64_SHR_U    => Op::I64_SHR_U    { dst, src1, src2 },
            Op2::I64_ROTL     => Op::I64_ROTL     { dst, src1, src2 },
            Op2::I64_ROTR     => Op::I64_ROTR     { dst, src1, src2 },
            Op2::F32_ADD      => Op::F32_ADD      { dst, src1, src2 },
            Op2::F32_SUB      => Op::F32_SUB      { dst, src1, src2 },
            Op2::F32_MUL      => Op::F32_MUL      { dst, src1, src2 },
            Op2::F32_DIV      => Op::F32_DIV      { dst, src1, src2 },
            Op2::F32_MIN      => Op::F32_MIN      { dst, src1, src2 },
            Op2::F32_MAX      => Op::F32_MAX      { dst, src1, src2 },
            Op2::F32_COPYSIGN => Op::F32_COPYSIGN { dst, src1, src2 },
            Op2::F64_ADD      => Op::F64_ADD      { dst, src1, src2 },
            Op2::F64_SUB      => Op::F64_SUB      { dst, src1, src2 },
            Op2::F64_MUL      => Op::F64_MUL      { dst, src1, src2 },
            Op2::F64_DIV      => Op::F64_DIV      { dst, src1, src2 },
            Op2::F64_MIN      => Op::F64_MIN      { dst, src1, src2 },
            Op2::F64_MAX      => Op::F64_MAX      { dst, src1, src2 },
            Op2::F64_COPYSIGN => Op::F64_COPYSIGN { dst, src1, src2 },
        }
    }


    // this seems to slow down the `Builder::build` quite a bit.
    // sadly it's not being implemented as a LUT.
    // if only you could use an attribute to control how the `match` is implemented...
    #[inline]
    pub fn num_extra_words(self) -> usize {
        use Op::*;
        match self {
            CALL_INDIRECT {..} | CALL_BYTECODE {..} | CALL_TABLE {..} |
            SELECT {..} |
            I32_CONST {..}  | F32_CONST {..}   |
            GLOBAL_GET {..} | GLOBAL_SET {..}  |
            LOAD_I32 {..}   | LOAD_I32_8S {..} | LOAD_I32_8U {..} | LOAD_I32_16S {..} | LOAD_I32_16U {..} |
            LOAD_I64 {..}   | LOAD_I64_8S {..} | LOAD_I64_8U {..} | LOAD_I64_16S {..} | LOAD_I64_16U {..} | LOAD_I64_32S {..} | LOAD_I64_32U {..} |
            LOAD_F32 {..}   | LOAD_F64 {..}    |
            STORE_I32 {..}  | STORE_I32_8 {..} | STORE_I32_16 {..} |
            STORE_I64 {..}  | STORE_I64_8 {..} | STORE_I64_16 {..} | STORE_I64_32 {..} |
            STORE_F32 {..}  | STORE_F64 {..}
            => 1,

            I64_CONST {..} | F64_CONST {..}
            => 2,

            JUMP_TABLE { len, .. } => len as usize,

            _ => 0,
        }
    }
}


#[derive(Clone, Copy)]
pub union Word {
    pub op:  Op,
    pub u32: u32,
}


pub struct Builder<RA: Alloc, TA: Alloc> {
    code:   Vec<Word, RA>,
    labels: Vec<u16,  TA>,
}

impl<RA: Alloc, TA: Alloc> Builder<RA, TA> {
    pub fn new(cap: usize, num_labels: usize, ra: RA, ta: TA) -> Self {
        let code = Vec::with_cap_in(ra, cap);

        let mut labels = Vec::with_cap_in(ta, num_labels);
        for _ in 0..num_labels {
            labels.push(u16::MAX);
        }

        Builder { code, labels }
    }

    #[inline]
    pub fn op(&mut self, op: Op) {
        self.code.push(Word { op });
    }

    #[inline]
    pub fn u32(&mut self, value: u32) {
        self.code.push(Word { u32: value });
    }

    #[inline]
    pub fn u64(&mut self, value: u64) {
        self.code.push(Word { u32: ((value >>  0) & 0xffffffff) as u32 });
        self.code.push(Word { u32: ((value >> 32) & 0xffffffff) as u32 });
    }

    #[inline]
    pub fn label(&mut self, label: u32) {
        self.labels[label as usize] = self.code.len() as u16;
    }

    pub fn build(self) -> Vec<Word, RA> {
        let mut code = self.code;

        if code.len() > i16::MAX as usize {
            unimplemented!()
        }


        // patch jumps.
        let mut pc = 0;
        while pc < code.len() {
            let prev_pc = pc;

            let op = unsafe { &mut code[pc].op };
            pc += 1 + op.num_extra_words();

            if let Op::JUMP_TABLE { len, .. } = *op {
                for i in 0..len as usize {
                    let word = &mut code[prev_pc + 1 + i];

                    let label = unsafe { word.u32 };

                    let target = self.labels[label as usize];
                    if target == u16::MAX {
                        unimplemented!()
                    }

                    let relative = target as i16 - pc as i16;
                    word.u32 = relative as u16 as u32;
                }

                continue;
            }

            use Op::*;
            match op {
                JUMP { delta } | JUMP_FALSE { delta, .. } | JUMP_TRUE { delta, .. } => {
                    let label = *delta as u16;

                    let target = self.labels[label as usize];
                    if target == u16::MAX {
                        unimplemented!()
                    }

                    let relative = target as i16 - pc as i16;
                    *delta = relative;
                }

                _ => (),
            }
        }

        return code;
    }
}

