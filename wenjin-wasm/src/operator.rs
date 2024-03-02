
macro_rules! for_each_operator {
    ($f:ident) => {
        $f! {
            Unreachable => visit_unreachable
            Nop => visit_nop
            Block { ty: $crate::BlockType } => visit_block
            Loop { yt: $crate::BlockType } => visit_loop
            If { ty: $crate::BlockType } => visit_if
            Else => visit_else
            End => visit_end
            Br { label: u32 } => visit_br
            BrIf { label: u32 }=> visit_br_if
            BrTable { table: () } => visit_br_table
            Return => visit_return
            Call { func: $crate::FuncIdx } => visit_call
            CallIndirect { ty: $crate::TypeIdx, table: $crate::TableIdx } => visit_call_indirect
            Drop => visit_drop
            Select => visit_select
            TypedSelect { ty: $crate::ValueType } => visit_select_ex
            LocalGet { idx: u32 } => visit_local_get
            LocalSet { idx: u32 } => visit_local_set
            LocalTee { idx: u32 } => visit_local_tee
            GlobalGet { idx: $crate::GlobalIdx } => visit_global_get
            GlobalSet { idx: $crate::GlobalIdx } => visit_global_set
            TableGet { idx: $crate::TableIdx } => visit_table_get
            TableSet { idx: $crate::TableIdx } => visit_table_set
            I32Load { align: u32, offset: u32 } => visit_i32_load
            I64Load { align: u32, offset: u32 } => visit_i64_load
            F32Load { align: u32, offset: u32 } => visit_f32_load
            F64Load { align: u32, offset: u32 } => visit_f64_load
            I32Load8S { align: u32, offset: u32 } => visit_i32_load8_s
            I32Load8U { align: u32, offset: u32 } => visit_i32_load8_u
            I32Load16S { align: u32, offset: u32 } => visit_i32_load16_s
            I32Load16U { align: u32, offset: u32 } => visit_i32_load16_u
            I64Load8S { align: u32, offset: u32 } => visit_i64_load8_s
            I64Load8U { align: u32, offset: u32 } => visit_i64_load8_u
            I64Load16S { align: u32, offset: u32 } => visit_i64_load16_s
            I64Load16U { align: u32, offset: u32 } => visit_i64_load16_u
            I64Load32S { align: u32, offset: u32 } => visit_i64_load32_s
            I64Load32U { align: u32, offset: u32 } => visit_i64_load32_u
            I32Store { align: u32, offset: u32 } => visit_i32_store
            I64Store { align: u32, offset: u32 } => visit_i64_store
            F32Store { align: u32, offset: u32 } => visit_f32_store
            F64Store { align: u32, offset: u32 } => visit_f64_store
            I32Store8 { align: u32, offset: u32 } => visit_i32_store8
            I32Store16 { align: u32, offset: u32 } => visit_i32_store16
            I64Store8 { align: u32, offset: u32 } => visit_i64_store8
            I64Store16 { align: u32, offset: u32 } => visit_i64_store16
            I64Store32 { align: u32, offset: u32 } => visit_i64_store32
            I32Const { value: i32 } => visit_i32_const
            I64Const { value: i64 } => visit_i64_const
            F32Const { value: f32 } => visit_f32_const
            F64Const { value: f64 } => visit_f64_const
            I32Eqz => visit_i32_eqz
            I32Eq => visit_i32_eq
            I32Ne => visit_i32_ne
            I32LtS => visit_i32_lt_s
            I32LtU => visit_i32_lt_u
            I32GtS => visit_i32_gt_s
            I32GtU => visit_i32_gt_u
            I32LeS => visit_i32_le_s
            I32LeU => visit_i32_le_u
            I32GeS => visit_i32_ge_s
            I32GeU => visit_i32_ge_u
            I64Eqz => visit_i64_eqz
            I64Eq => visit_i64_eq
            I64Ne => visit_i64_ne
            I64LtS => visit_i64_lt_s
            I64LtU => visit_i64_lt_u
            I64GtS => visit_i64_gt_s
            I64GtU => visit_i64_gt_u
            I64LeS => visit_i64_le_s
            I64LeU => visit_i64_le_u
            I64GeS => visit_i64_ge_s
            I64GeU => visit_i64_ge_u
            F32Eq => visit_f32_eq
            F32Ne => visit_f32_ne
            F32Lt => visit_f32_lt
            F32Gt => visit_f32_gt
            F32Le => visit_f32_le
            F32Ge => visit_f32_ge
            F64Eq => visit_f64_eq
            F64Ne => visit_f64_ne
            F64Lt => visit_f64_lt
            F64Gt => visit_f64_gt
            F64Le => visit_f64_le
            F64Ge => visit_f64_ge
            I32Clz => visit_i32_clz
            I32Ctz => visit_i32_ctz
            I32Popcnt => visit_i32_popcnt
            I32Add => visit_i32_add
            I32Sub => visit_i32_sub
            I32Mul => visit_i32_mul
            I32DivS => visit_i32_div_s
            I32DivU => visit_i32_div_u
            I32RemS => visit_i32_rem_s
            I32RemU => visit_i32_rem_u
            I32And => visit_i32_and
            I32Or => visit_i32_or
            I32Xor => visit_i32_xor
            I32Shl => visit_i32_shl
            I32ShrS => visit_i32_shr_s
            I32ShrU => visit_i32_shr_u
            I32Rotl => visit_i32_rotl
            I32Rotr => visit_i32_rotr
            I64Clz => visit_i64_clz
            I64Ctz => visit_i64_ctz
            I64Popcnt => visit_i64_popcnt
            I64Add => visit_i64_add
            I64Sub => visit_i64_sub
            I64Mul => visit_i64_mul
            I64DivS => visit_i64_div_s
            I64DivU => visit_i64_div_u
            I64RemS => visit_i64_rem_s
            I64RemU => visit_i64_rem_u
            I64And => visit_i64_and
            I64Or => visit_i64_or
            I64Xor => visit_i64_xor
            I64Shl => visit_i64_shl
            I64ShrS => visit_i64_shr_s
            I64ShrU => visit_i64_shr_u
            I64Rotl => visit_i64_rotl
            I64Rotr => visit_i64_rotr
            F32Abs => visit_f32_abs
            F32Neg => visit_f32_neg
            F32Ceil => visit_f32_ceil
            F32Floor => visit_f32_floor
            F32Trunc => visit_f32_trunc
            F32Nearest => visit_f32_nearest
            F32Sqrt => visit_f32_sqrt
            F32Add => visit_f32_add
            F32Sub => visit_f32_sub
            F32Mul => visit_f32_mul
            F32Div => visit_f32_div
            F32Min => visit_f32_min
            F32Max => visit_f32_max
            F32Copysign => visit_f32_copysign
            F64Abs => visit_f64_abs
            F64Neg => visit_f64_neg
            F64Ceil => visit_f64_ceil
            F64Floor => visit_f64_floor
            F64Trunc => visit_f64_trunc
            F64Nearest => visit_f64_nearest
            F64Sqrt => visit_f64_sqrt
            F64Add => visit_f64_add
            F64Sub => visit_f64_sub
            F64Mul => visit_f64_mul
            F64Div => visit_f64_div
            F64Min => visit_f64_min
            F64Max => visit_f64_max
            F64Copysign => visit_f64_copysign
            I32WrapI64 => visit_i32_wrap_i64
            I32TruncF32S => visit_i32_trunc_f32_s
            I32TruncF32U => visit_i32_trunc_f32_u
            I32TruncF64S => visit_i32_trunc_f64_s
            I32TruncF64U => visit_i32_trunc_f64_u
            I64ExtendI32S => visit_i64_extend_i32_s
            I64ExtendI32U => visit_i64_extend_i32_u
            I64TruncF32S => visit_i64_trunc_f32_s
            I64TruncF32U => visit_i64_trunc_f32_u
            I64TruncF64S => visit_i64_trunc_f64_s
            I64TruncF64U => visit_i64_trunc_f64_u
            F32ConvertI32S => visit_f32_convert_i32_s
            F32ConvertI32U => visit_f32_convert_i32_u
            F32ConvertI64S => visit_f32_convert_i64_s
            F32ConvertI64U => visit_f32_convert_i64_u
            F32DemoteF64 => visit_f32_demote_f64
            F64ConvertI32S => visit_f64_convert_i32_s
            F64ConvertI32U => visit_f64_convert_i32_u
            F64ConvertI64S => visit_f64_convert_i64_s
            F64ConvertI64U => visit_f64_convert_i64_u
            F64PromoteF32 => visit_f64_promote_f32
            I32ReinterpretF32 => visit_i32_reinterpret_f32
            I64ReinterpretF64 => visit_i64_reinterpret_f64
            F32ReinterpretI32 => visit_f32_reinterpret_i32
            F64ReinterpretI64 => visit_f64_reinterpret_i64
            I32Extend8S => visit_i32_extend8_s
            I32Extend16S => visit_i32_extend16_s
            I64Extend8S => visit_i64_extend8_s
            I64Extend16S => visit_i64_extend16_s
            I64Extend32S => visit_i64_extend32_s
            RefNull => visit_ref_null
            RefIsNull => visit_ref_is_null
            RefFunc => visit_ref_func
            MemorySize { mem: $crate::MemoryIdx } => visit_memory_size
            MemoryGrow { mem: $crate::MemoryIdx } => visit_memory_grow
            MemoryCopy { dst: $crate::MemoryIdx, src: $crate::MemoryIdx } => visit_memory_copy
            MemoryFill { mem: $crate::MemoryIdx } => visit_memory_fill
        }
    };
}


macro_rules! operator_enum {
    ($($op:ident $({ $($arg:ident: $argty:ty),* })? => $visitor:ident)*) => {
        pub enum Operator {
            $($op $({ $($arg: $argty),* })?,)*
        }
    };
}
for_each_operator!(operator_enum);


macro_rules! operator_visitor {
    ($($op:ident $({ $($arg:ident: $argty:ty),* })? => $visitor:ident)*) => {
        pub trait OperatorVisitor {
            type Output;

            $(fn $visitor(&mut self, $($($arg: $argty),*)?) -> Self::Output;)*
        }
    };
}
for_each_operator!(operator_visitor);


pub struct NewOperator;

macro_rules! new_operator {
    ($($op:ident $({ $($arg:ident: $argty:ty),* })? => $visitor:ident)*) => {
        impl OperatorVisitor for NewOperator {
            type Output = Operator;

            $(fn $visitor(&mut self, $($($arg: $argty),*)?) -> Self::Output {
                Operator::$op $({ $($arg),* })?
            })*
        }
    };
}
for_each_operator!(new_operator);


