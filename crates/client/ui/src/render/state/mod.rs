use rimecraft_render::ProvideRenderTy;
use rimecraft_voxel_math::BBox;

use crate::{render::item::ItemDisplayMode, screen::ScreenRect};

pub struct Layer<'l> {
    pub parent: Option<&'l Layer<'l>>,
}

pub trait ElementState {
    fn bounds(&self) -> Option<ScreenRect>;
}

pub trait SimpleElementState<Cx>: ElementState
where
    Cx: ProvideRenderTy,
{
    fn fill_vertices(&mut self, consumer: &mut Cx::VertexConsumer, depth: f32);

    fn pipeline(&self) -> &Cx::Pipeline;

    fn texture_setup(&self) -> &Cx::TextureSetup;

    fn scissor_rect(&self) -> Option<ScreenRect>;
}

pub struct ItemElementState {
    mode: Option<ItemDisplayMode>,
    layer_count: u32,
    is_animated: bool,
    is_oversized: bool,
    cached_model_bounds: Option<BBox>,
}
