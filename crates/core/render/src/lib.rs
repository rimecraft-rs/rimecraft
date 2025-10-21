//! Minecraft rendering API.

use rimecraft_global_cx::GlobalContext;

use crate::{pipeline::Pipeline, vertex::VertexConsumer};

pub mod draw;
pub mod pipeline;
pub mod vertex;

pub trait ProvideRenderTy: GlobalContext {
    type VertexConsumer: VertexConsumer;
    type Pipeline: Pipeline;
    type TextureSetup: TextureSetup;
}

pub trait TextureSetup {}
