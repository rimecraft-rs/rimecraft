use super::*;

pub struct SuppliableIntCallbacks {
    min_boundary: fn() -> i32,
    max_boundary: fn() -> i32,
}

impl<Txt> TypeChangeableCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: Texts,
{
    fn is_cycling(&self) -> bool {
        true
    }
}

impl<Txt> CyclingCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: Texts,
{
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<Txt> IntSliderCallbacks<Txt> for SuppliableIntCallbacks
where
    Txt: Texts,
{
    fn min_inclusive(&self) -> i32 {
        (self.min_boundary)()
    }

    fn max_inclusive(&self) -> i32 {
        (self.max_boundary)()
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        match value {
            Some(value) => Some(rimecraft_math::clamp(
                value as f32,
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::min_inclusive(self) as f32,
                <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::max_inclusive(self) as f32,
            ) as i32),
            None => None,
        }
    }
}

impl<Txt> SliderCallbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: Texts,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::to_value(self, slider_progress)
    }
}

impl<Txt> Callbacks<i32, Txt> for SuppliableIntCallbacks
where
    Txt: Texts,
{
    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<i32>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: &ChangeCallback<i32>,
    ) -> WidgetCreator<i32, Txt> {
        <SuppliableIntCallbacks as SliderCallbacks<i32, Txt>>::get_widget_creator(
            self,
            tooltip_factory,
            game_options,
            x,
            y,
            width,
            change_callback,
        )
    }

    fn validate(&self, value: Option<i32>) -> Option<i32> {
        <SuppliableIntCallbacks as IntSliderCallbacks<Txt>>::i32_validate(self, value)
    }
}