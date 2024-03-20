use sti::traits::CopyIt;
use sti::vec::Vec;
use sti::hash::HashMap;

use crate::{Result, Error, ErrorKind};
use crate::{ValueType, BlockType, TypeIdx, FuncIdx, TableIdx, MemoryIdx, GlobalIdx, Module, TableType, FuncType, RefType, GlobalType, MemoryType};


pub const DEFAULT_STACK_LIMIT: u32 = 1024;
pub const DEFAULT_FRAME_LIMIT: u32 = 4096;

pub struct Validator<'a> {
    pub stack_limit: u32,
    pub frame_limit: u32,

    pub module: &'a Module<'a>,

    pos: usize,

    locals: Vec<ValueType>,
    stack: Vec<ValueType>,
    frames: Vec<Frame>,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    kind:        FrameKind,
    ty:          BlockType,
    height:      u32,
    unreachable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameKind {
    Block { last_use: u32, },
    If    { this: u32, last_use: u32 },
    Else  { last_use: u32 },
    Loop  { this: u32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Jump {
    pub target: u32,
    pub shift_num: u32,
    pub shift_by: u32,
}


impl<'a> Validator<'a> {
    pub fn new(module: &'a Module<'a>) -> Self {
        Self {
            stack_limit: DEFAULT_STACK_LIMIT,
            frame_limit: DEFAULT_FRAME_LIMIT,
            module,
            pos: 0,
            locals: Vec::new(),
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn locals(&self) -> &[ValueType] {
        &self.locals
    }

    #[inline(always)]
    pub fn num_locals(&self) -> u32 {
        self.locals.len() as u32
    }

    #[inline(always)]
    pub fn stack(&self) -> &[ValueType] {
        &self.stack
    }

    #[inline(always)]
    pub fn num_stack(&self) -> u32 {
        self.stack.len() as u32
    }

    #[inline(always)]
    pub fn num_frames(&self) -> u32 {
        self.frames.len() as u32
    }


    pub fn is_unreachable(&self) -> bool {
        self.frames.rev(0).unreachable
    }

    fn unreachable(&mut self) {
        let frame = self.frames.rev_mut(0);
        frame.unreachable = true;
        self.stack.truncate(frame.height as usize);
    }


    fn push(&mut self, ty: ValueType) -> Result<()> {
        if !self.is_unreachable() {
            if self.stack.len() >= self.stack_limit as usize {
                return Err(self.error(ErrorKind::StackLimit));
            }

            self.stack.push(ty);
        }
        return Ok(());
    }

    fn push_n(&mut self, tys: &[ValueType]) -> Result<()> {
        for ty in tys {
            self.push(*ty)?;
        }
        return Ok(());
    }

    fn pop(&mut self) -> Result<ValueType> {
        debug_assert!(!self.is_unreachable());

        if self.stack.len() <= self.frames.rev(0).height as usize {
            return Err(self.error(ErrorKind::StackUnderflow));
        }

        return Ok(self.stack.pop().unwrap());
    }

    fn expect(&mut self, expected_ty: ValueType) -> Result<()> {
        if !self.is_unreachable() {
            if self.stack.len() <= self.frames.rev(0).height as usize {
                return Err(self.error(ErrorKind::StackUnderflow));
            }

            let ty = self.stack.pop().unwrap();
            if ty != expected_ty {
                return Err(self.error(ErrorKind::TypeMismatch { expected: expected_ty, found: ty }));
            }
        }
        return Ok(());
    }

    fn expect_n(&mut self, expected_tys: &[ValueType]) -> Result<()> {
        for ty in expected_tys.iter().rev() {
            self.expect(*ty)?;
        }
        Ok(())
    }


    // pushes the block begin types.
    fn push_frame(&mut self, kind: FrameKind, ty: BlockType) -> Result<()> {
        if self.frames.len() >= self.frame_limit as usize {
            return Err(self.error(ErrorKind::FrameLimit));
        }

        let height = self.stack.len() as u32;

        self.push_n(self.block_begin_types(ty))?;

        self.frames.push(Frame {
            kind,
            ty,
            height,
            unreachable: self.is_unreachable(),
        });

        return Ok(());
    }

    // expects the block end types.
    fn pop_frame(&mut self) -> Result<Frame> {
        let Some(frame) = self.frames.last().copied() else {
            return Err(self.error(ErrorKind::UnexpectedEnd));
        };

        self.expect_n(self.block_end_types(frame.ty))?;
        if self.stack.len() != frame.height as usize {
            return Err(self.error(ErrorKind::FrameExtraStack));
        }

        let frame = self.frames.pop().unwrap();
        return Ok(frame);
    }


    fn label(&self, idx: u32) -> Result<Frame> {
        // @todo: get_rev
        let idx = idx as usize;
        if idx >= self.frames.len() {
            return Err(self.error(ErrorKind::InvalidLabel));
        }
        Ok(*self.frames.rev(idx))
    }

    fn ty(&self, idx: TypeIdx) -> Result<FuncType<'a>> {
        self.module.types.get(idx as usize).copied()
            .ok_or_else(|| self.error(ErrorKind::InvalidTypeIdx))
    }

    fn local(&self, idx: u32) -> Result<ValueType> {
        self.locals.get(idx as usize).copied()
            .ok_or_else(|| self.error(ErrorKind::InvalidLocalIdx))
    }

    fn func(&self, idx: FuncIdx) -> Result<FuncType<'a>> {
        let type_idx = self.module.get_func(idx)
            .ok_or_else(|| self.error(ErrorKind::InvalidFuncIdx))?;
        // by imports-valid, funcs-valid.
        Ok(self.module.types[type_idx as usize])
    }

    fn table(&self, idx: TableIdx) -> Result<TableType> {
        self.module.get_table(idx)
            .ok_or_else(|| self.error(ErrorKind::InvalidTableIdx))
    }

    fn memory(&self, idx: MemoryIdx) -> Result<MemoryType> {
        self.module.get_memory(idx)
            .ok_or_else(|| self.error(ErrorKind::InvalidMemoryIdx))
    }

    fn global(&self, idx: GlobalIdx) -> Result<GlobalType> {
        self.module.get_global(idx)
            .ok_or_else(|| self.error(ErrorKind::InvalidGlobalIdx))
    }


    #[inline]
    fn block_begin_types(&self, ty: BlockType) -> &'a [ValueType] {
        ty.begin_types(self.module)
    }

    #[inline]
    fn block_end_types(&self, ty: BlockType) -> &'a [ValueType] {
        ty.end_types(self.module)
    }

    #[inline]
    fn frame_br_types(&self, frame: &Frame) -> &'a [ValueType] {
        if matches!(frame.kind, FrameKind::Loop {..}) {
            // br in a loop means continue.
            // so we need the initial types again.
            self.block_begin_types(frame.ty)
        }
        else {
            // otherwise, break.
            self.block_end_types(frame.ty)
        }
    }


    pub fn validate_func
        (&mut self,
         parser: &mut crate::Parser,
         func_ty: TypeIdx,
         locals: &[ValueType],
         mut jumps: Option<&mut HashMap<u32, Jump>>)
        -> Result<()>
    {
        let begin_func = parser.reader.offset();

        self.locals.truncate(0);
        let params = self.ty(func_ty)?.params;
        self.locals.reserve(params.len() + locals.len());
        for param in params.copy_it() {
            self.locals.push(param);
        }
        for local in locals.copy_it() {
            self.locals.push(local);
        }

        self.stack.truncate(0);

        self.frames.truncate(0);
        self.frames.push(Frame {
            kind: FrameKind::Block { last_use: u32::MAX },
            ty: BlockType::Func(func_ty),
            height: 0,
            unreachable: false,
        });

        fn jump(this: &mut Validator, from: u32, to: u32, num_rets: usize, jumps: &mut HashMap<u32, Jump>) {
            let stack = this.num_stack();
            let frame = this.frames.rev_mut(to as usize);

            let shift_num = num_rets as u32;
            let shift_by = stack - frame.height;

            let target = match &mut frame.kind {
                FrameKind::Block {last_use} |
                FrameKind::If {last_use, ..} |
                FrameKind::Else {last_use} => {
                    let prev = *last_use;
                    *last_use = from;
                    prev
                }

                FrameKind::Loop { this } => {
                    *this
                }
            };

            let none = jumps.insert(from, Jump { target, shift_num, shift_by });
            debug_assert!(none.is_none());
        }

        fn patch_jumps(jumps: &mut HashMap<u32, Jump>, last_use: u32, dst: u32) {
            let mut at = last_use;
            while at != u32::MAX {
                let entry = &mut jumps[&at];
                at = entry.target;
                entry.target = dst;
            }
        }

        while !parser.is_done() {
            self.pos = parser.reader.offset();
            let opcode = parser.parse_opcode()?;

            use crate::opcode::OpcodeClass;
            match opcode.class() {
                OpcodeClass::Basic { pop, push } => {
                    self.expect_n(pop)?;
                    self.push_n(push)?;
                }

                OpcodeClass::Mem { max_align, pop, push } => {
                    let align = parser.parse_u32()?;
                    let _offset = parser.parse_u32()?;

                    self.memory(0)?;

                    let align = 1u32.checked_shl(align)
                        .ok_or_else(|| self.error(ErrorKind::AlignTooLarge))?;
                    if align > max_align as u32 {
                        return Err(self.error(ErrorKind::AlignTooLarge));
                    }

                    self.expect_n(pop)?;
                    self.push_n(push)?;
                }

                OpcodeClass::Unreachable => {
                    self.unreachable();
                }

                OpcodeClass::Block => {
                    let ty = parser.parse_block_type()?;
                    self.expect_n(self.block_begin_types(ty))?;
                    self.push_frame(FrameKind::Block { last_use: u32::MAX }, ty)?;
                }

                OpcodeClass::Loop => {
                    let ty = parser.parse_block_type()?;
                    self.expect_n(self.block_begin_types(ty))?;
                    let this = (self.pos - begin_func) as u32;
                    self.push_frame(FrameKind::Loop { this }, ty)?;
                }

                OpcodeClass::If => {
                    let ty = parser.parse_block_type()?;
                    self.expect(ValueType::I32)?;
                    self.expect_n(self.block_begin_types(ty))?;
                    let this = (self.pos - begin_func) as u32;
                    self.push_frame(FrameKind::If { this, last_use: u32::MAX }, ty)?;
                }

                OpcodeClass::Else => {
                    if self.frames.is_empty() {
                        return Err(self.error(ErrorKind::UnexpectedElse));
                    }

                    let frame = self.pop_frame()?;
                    let FrameKind::If { this: the_if, last_use } = frame.kind else {
                        return Err(self.error(ErrorKind::UnexpectedElse));
                    };

                    let this = (self.pos - begin_func) as u32;
                    if let Some(jumps) = jumps.as_mut() {
                        jumps.insert(the_if, Jump { target: this, shift_num: 0, shift_by: 0 });
                    }

                    self.push_frame(FrameKind::Else { last_use }, frame.ty)?;
                }

                OpcodeClass::End => {
                    let frame = self.pop_frame()?;

                    let this = (self.pos - begin_func) as u32;
                    match frame.kind {
                        FrameKind::Block { last_use } |
                        FrameKind::Else { last_use } => {
                            if let Some(jumps) = &mut jumps {
                                patch_jumps(jumps, last_use, this);
                            }
                        }

                        FrameKind::If { this: the_if, last_use } => {
                            let begin_types = self.block_begin_types(frame.ty);
                            let end_types = self.block_end_types(frame.ty);
                            if end_types != begin_types {
                                return Err(self.error(ErrorKind::NonIdIfWithoutElse));
                            }

                            if let Some(jumps) = &mut jumps {
                                jumps.insert(the_if, Jump { target: this, shift_num: 0, shift_by: 0 });
                                patch_jumps(jumps, last_use, this);
                            }
                        }


                        FrameKind::Loop {..} => ()
                    }

                    if self.frames.len() > 0 {
                        self.push_n(self.block_end_types(frame.ty))?;
                    }
                    else { break }
                }

                OpcodeClass::Br => {
                    let label = parser.parse_label()?;
                    let frame = self.label(label)?;

                    let tys = self.frame_br_types(&frame);
                    self.expect_n(tys)?;

                    if let Some(jumps) = &mut jumps {
                        let this = (self.pos - begin_func) as u32;
                        jump(self, this, label, tys.len(), jumps);
                    }

                    self.unreachable();
                }

                OpcodeClass::BrIf => {
                    let label = parser.parse_label()?;
                    let frame = self.label(label)?;
                    self.expect(ValueType::I32)?;

                    let tys = self.frame_br_types(&frame);
                    self.expect_n(tys)?;

                    if let Some(jumps) = &mut jumps {
                        let this = (self.pos - begin_func) as u32;
                        jump(self, this, label, tys.len(), jumps);
                    }

                    self.push_n(tys)?;
                }

                OpcodeClass::BrTable => {
                    let table = parser.parse_br_table()?;
                    let frame = self.label(table.default)?;

                    let tys = self.frame_br_types(&frame);
                    self.expect(ValueType::I32)?;
                    self.expect_n(tys)?;

                    let mut this = (self.pos - begin_func) as u32;
                    if let Some(jumps) = &mut jumps {
                        jump(self, this, table.default, tys.len(), jumps);
                    }

                    for label in table.labels() {
                        let f = self.label(label)?;
                        if !self.is_unreachable() && self.frame_br_types(&f) != tys {
                            return Err(self.error(ErrorKind::BrTableInvalidTargetTypes { label }));
                        }

                        if let Some(jumps) = &mut jumps {
                            this += 1;
                            jump(self, this, label, tys.len(), jumps);
                        }
                    }

                    self.unreachable();
                }

                OpcodeClass::Return => {
                    let frame = self.frames[0];
                    self.expect_n(self.block_end_types(frame.ty))?;
                    self.unreachable();
                }

                OpcodeClass::Call => {
                    let func = parser.parse_func_idx()?;
                    let ty = self.func(func)?;
                    self.expect_n(ty.params)?;
                    self.push_n(ty.rets)?
                }

                OpcodeClass::CallIndirect => {
                    let ty = parser.parse_type_idx()?;
                    let table = parser.parse_table_idx()?;

                    let table = self.table(table)?;
                    if table.ty != RefType::FuncRef {
                        return Err(self.error(ErrorKind::CallIndirectTableNotOfFuncRefs));
                    }

                    let ty = self.ty(ty)?;
                    self.expect(ValueType::I32)?;
                    self.expect_n(ty.params)?;
                    self.push_n(ty.rets)?;
                }

                OpcodeClass::Drop => {
                    if !self.is_unreachable() {
                        self.pop()?;
                    }
                }

                OpcodeClass::Select => {
                    self.expect(ValueType::I32)?;

                    if !self.is_unreachable() {
                        let t1 = self.pop()?;
                        let t2 = self.pop()?;

                        if t1.is_ref() || t2.is_ref() {
                            return Err(self.error(ErrorKind::SelectUnexpectedRefType));
                        }

                        if t1 != t2 {
                            return Err(self.error(ErrorKind::SelectTypeMismatch(t1, t2)));
                        }

                        self.push(t1)?;
                    }
                }

                OpcodeClass::TypedSelect => {
                    let ty = parser.parse_typed_select()?;
                    self.expect(ValueType::I32)?;
                    self.expect(ty)?;
                    self.expect(ty)?;
                    self.push(ty)?;
                }

                OpcodeClass::LocalGet => {
                    let idx = parser.parse_local_idx()?;
                    let ty = self.local(idx)?;
                    self.push(ty)?;
                }

                OpcodeClass::LocalSet => {
                    let idx = parser.parse_local_idx()?;
                    let ty = self.local(idx)?;
                    self.expect(ty)?;
                }

                OpcodeClass::LocalTee => {
                    let idx = parser.parse_local_idx()?;
                    let ty = self.local(idx)?;
                    self.expect(ty)?;
                    self.push(ty)?;
                }

                OpcodeClass::GlobalGet => {
                    let idx = parser.parse_global_idx()?;
                    let g = self.global(idx)?;
                    self.push(g.ty)?;
                }

                OpcodeClass::GlobalSet => {
                    let idx = parser.parse_global_idx()?;
                    let g = self.global(idx)?;
                    if !g.mutable {
                        return Err(self.error(ErrorKind::GlobalNotMutable));
                    }
                    self.expect(g.ty)?;
                }

                OpcodeClass::TableGet => {
                    let idx = parser.parse_u32()?;
                    let _ = idx;
                    return Err(self.error(ErrorKind::Todo));
                }

                OpcodeClass::TableSet => {
                    let idx = parser.parse_u32()?;
                    let _ = idx;
                    return Err(self.error(ErrorKind::Todo));
                }

                OpcodeClass::MemorySize => {
                    let mem = parser.parse_memory_idx()?;
                    self.memory(mem)?;
                    self.push(ValueType::I32)?;
                }

                OpcodeClass::MemoryGrow => {
                    let mem = parser.parse_memory_idx()?;
                    self.memory(mem)?;
                    self.expect(ValueType::I32)?;
                    self.push(ValueType::I32)?;
                }

                OpcodeClass::I32Const => {
                    let _ = parser.parse_i32()?;
                    self.push(ValueType::I32)?;
                }

                OpcodeClass::I64Const => {
                    let _ = parser.parse_i64()?;
                    self.push(ValueType::I64)?;
                }

                OpcodeClass::F32Const => {
                    let _ = parser.parse_f32()?;
                    self.push(ValueType::F32)?;
                }

                OpcodeClass::F64Const => {
                    let _ = parser.parse_f64()?;
                    self.push(ValueType::F64)?;
                }

                OpcodeClass::RefNull => {
                    let ty = parser.parse_ref_type()?;
                    self.push(ty.to_value_type())?;
                }

                OpcodeClass::RefIsNull => {
                    if !self.is_unreachable() {
                        let ty = self.pop()?;
                        match ty {
                            ValueType::FuncRef |
                            ValueType::ExternRef => (),

                            ValueType::I32 |
                            ValueType::I64 |
                            ValueType::F32 |
                            ValueType::F64 |
                            ValueType::V128 => return Err(self.error(
                                ErrorKind::RefTypeExpected { found: ty }))
                        }

                        self.push(ValueType::I32)?;
                    }
                }

                OpcodeClass::RefFunc => {
                    return Err(self.error(ErrorKind::Todo));
                }

                OpcodeClass::MemoryCopy => {
                    let dst = parser.parse_memory_idx()?;
                    let src = parser.parse_memory_idx()?;
                    self.memory(dst)?;
                    self.memory(src)?;
                    self.expect(ValueType::I32)?;
                    self.expect(ValueType::I32)?;
                    self.expect(ValueType::I32)?;
                }

                OpcodeClass::MemoryFill => {
                    let mem = parser.parse_memory_idx()?;
                    self.memory(mem)?;
                    self.expect(ValueType::I32)?;
                    self.expect(ValueType::I32)?;
                    self.expect(ValueType::I32)?;
                }
            }
        }
        self.pos = parser.reader.offset();

        if !parser.is_done() {
            todo!()
        }

        if self.frames.len() != 0 {
            return Err(self.error(ErrorKind::MissingEnd));
        }
        assert_eq!(self.stack.len(), 0);

        return Ok(());
    }

    #[inline]
    fn error(&self, kind: ErrorKind) -> Error {
        Error { pos: self.pos, kind }
    }
}

