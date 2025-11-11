use rimecraft_render_math::screen::ScreenSize;

use crate::layout::{LayoutPack, LayoutValue};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SizeConstraint {
    #[default]
    Free,
    Fixed(LayoutValue),
    Flexible {
        min: Option<LayoutValue>,
        max: Option<LayoutValue>,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SizeConstraints<Ext> {
    pub horizontal: SizeConstraint,
    pub vertical: SizeConstraint,
    pub ext: Ext,
}

impl<Ext> SizeConstraints<Ext> {
    pub fn new(horizontal: SizeConstraint, vertical: SizeConstraint, ext: Ext) -> Self {
        Self {
            horizontal,
            vertical,
            ext,
        }
    }

    pub fn free(ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Free,
            vertical: SizeConstraint::Free,
            ext,
        }
    }

    pub fn fixed(horizontal: LayoutValue, vertical: LayoutValue, ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Fixed(horizontal),
            vertical: SizeConstraint::Fixed(vertical),
            ext,
        }
    }

    pub fn from_size(size: ScreenSize, ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Fixed(LayoutValue::Px(size.horizontal)),
            vertical: SizeConstraint::Fixed(LayoutValue::Px(size.vertical)),
            ext,
        }
    }
}

impl<Ext> SizeConstraints<Ext> {
    pub fn minimum_size(&self) -> LayoutPack<Option<LayoutValue>> {
        let horizontal = match self.horizontal {
            SizeConstraint::Free => Some(LayoutValue::Px(0.0)),
            SizeConstraint::Fixed(h) => Some(h),
            SizeConstraint::Flexible { min: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { min: None, .. } => Some(LayoutValue::Px(0.0)),
        };
        let vertical = match self.vertical {
            SizeConstraint::Free => Some(LayoutValue::Px(0.0)),
            SizeConstraint::Fixed(v) => Some(v),
            SizeConstraint::Flexible { min: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { min: None, .. } => Some(LayoutValue::Px(0.0)),
        };
        LayoutPack::new(horizontal, vertical)
    }

    pub fn maximum_size(&self) -> LayoutPack<Option<LayoutValue>> {
        let horizontal = match self.horizontal {
            SizeConstraint::Free => None,
            SizeConstraint::Fixed(h) => Some(h),
            SizeConstraint::Flexible { max: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { max: None, .. } => None,
        };
        let vertical = match self.vertical {
            SizeConstraint::Free => None,
            SizeConstraint::Fixed(v) => Some(v),
            SizeConstraint::Flexible { max: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { max: None, .. } => None,
        };
        LayoutPack::new(horizontal, vertical)
    }
}
