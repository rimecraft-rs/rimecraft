use super::NbtElement;

pub trait NbtElementVisitor {
    fn visit(&mut self, element: &NbtElement);
}
