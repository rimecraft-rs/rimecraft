use std::{any::TypeId, hash::Hash};

use crate::prelude::*;

pub struct BlockEntity {
    be_type: Type,
    data: Box<dyn Data>,
    pos: BlockPos,
    state: super::SharedBlockState,
}

impl BlockEntity {
    pub fn data<T: std::any::Any>(&self) -> &T {
        assert_eq!(TypeId::of::<T>(), self.data.type_id());
        unsafe { &*(&*self.data as *const dyn Data as *const T) }
    }

    pub fn data_mut<T: std::any::Any>(&mut self) -> &mut T {
        assert_eq!(TypeId::of::<T>(), self.data.type_id());
        unsafe { &mut *(&mut *self.data as *mut dyn Data as *mut T) }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn get_state(&self) -> super::SharedBlockState {
        self.state
    }

    pub fn get_type(&self) -> Type {
        self.be_type
    }

    fn read_nbt(&mut self, nbt: &rimecraft_nbt_ext::Compound) {
        self.data.read(nbt)
    }

    fn write_nbt(&self, nbt: &mut rimecraft_nbt_ext::Compound) {
        self.data.write(nbt)
    }
}

pub trait Data: std::any::Any + Send + Sync + 'static {
    fn read(&mut self, nbt: &rimecraft_nbt_ext::Compound);

    fn write(&self, nbt: &mut rimecraft_nbt_ext::Compound);
}

#[derive(Clone, Copy, Eq)]
pub struct Type {
    id: usize,
    blocks: rimecraft_primitives::Ref<'static, Vec<super::Block>>,
}

impl Type {
    /// Returns whether the block entity type supports `state`.
    pub fn supports(&self, state: &super::BlockState) -> bool {
        self.blocks.contains(&state.block())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id)
    }
}

impl crate::registry::Registration for Type {
    fn accept(&mut self, id: usize) {
        self.id = id
    }

    fn index_of(&self) -> usize {
        self.id
    }
}

pub trait Provide {
    //TODO world

    fn create(&self, pos: BlockPos, state: &super::BlockState) -> Option<Box<dyn Data>>;

    fn ticker(&self, state: &super::BlockState, be_type: Type) -> Option<Ticker> {
        None
    }
}

pub type Ticker = fn((), BlockPos, &super::BlockState, &mut BlockEntity);
