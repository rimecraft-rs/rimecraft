//! Chunk managers.

use crate::chunk::{Chunk, ChunkCx};

pub trait ChunkView<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type Chunk: Chunk<'w, Cx>;
}
