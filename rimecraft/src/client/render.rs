use glium::vertex;
use std::borrow::Cow;

pub struct VertexFormatElementBuilder {
    name: Cow<'static, str>,
    attr_type: vertex::AttributeType,
}

pub trait VertexConsumer {
    fn vertex(&mut self, x: f64, y: f64, z: f64);
    fn color(&mut self, r: u32, g: u32, b: u32, a: u32);
    fn texture(&mut self, var1: f32, var2: f32);
}
