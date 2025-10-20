//! Vertex building traits.

/// A vertex consumer for building vertices.
pub trait VertexConsumer {
    /// The chainnable vertex builder type.
    type Chain: VertexChain;

    /// Begins building a new vertex using the chainnable builder.
    fn begin(&mut self) -> Self::Chain;
}

/// A chainable vertex builder.
pub trait VertexChain {
    /// Adds position attribute.
    fn pos(self, pos: impl Into<glam::Vec3A>) -> Self;

    /// Adds color attribute.
    fn color(self, color: impl Into<glam::Vec4>) -> Self;

    /// Adds texture coordinate attribute.
    fn uv_tex(self, uv: impl Into<glam::Vec2>) -> Self;

    /// Adds overlay texture coordinate attribute.
    fn uv_overlay(self, uv: impl Into<glam::Vec2>) -> Self;

    /// Adds lightmap texture coordinate attribute.
    fn uv_light(self, uv: impl Into<glam::Vec2>) -> Self;

    /// Adds normal attribute.
    fn norm(self, norm: impl Into<glam::Vec3A>) -> Self;

    /// Finalizes the vertex.
    fn finish(self);
}
