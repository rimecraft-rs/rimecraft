//! UI widgets.

use rimecraft_local_cx::LocalContext;
use rimecraft_render::draw::{Drawable, ProvideDrawTy};

use crate::{
    Element, MeasurableElement, ProvideUiTy, Selectable,
    layout::{pos::PosConstraints, size::SizeConstraints},
};

pub trait Widget<Cx>
where
    Cx: ProvideUiTy,
{
    fn set_pos_constraints(&self, constraints: PosConstraints<Cx::PosConstraintsExt>);

    fn set_size_constraints(&self, constraints: SizeConstraints<Cx::SizeConstraintsExt>);

    fn interactable_widgets(&self) -> Cx::InteractableWidgetIter<'_>;
}

pub trait InteractableWidget<'a, Cx>:
    Drawable<'a, Cx> + Element<Cx> + Widget<Cx> + Selectable
where
    Cx: ProvideUiTy + ProvideDrawTy,
{
}
