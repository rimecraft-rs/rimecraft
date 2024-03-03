use super::*;

struct ValidatingIntSliderCallbacks {
    min: i32,
    max: i32,
}

impl<Txt> IntSliderCallbacks<Txt> for ValidatingIntSliderCallbacks
where
    Txt: Texts,
{
    fn min_inclusive(&self) -> i32 {
        self.min
    }

    fn max_inclusive(&self) -> i32 {
        self.max
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32> {
        match value {
            Some(value) => {
                if rimecraft_math::in_range(value as f32, self.min as f32, self.max as f32) {
                    Some(value)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl<Txt> SliderCallbacks<i32, Txt> for ValidatingIntSliderCallbacks
where
    Txt: Texts,
{
    fn to_slider_progress(&self, value: i32) -> f32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::to_slider_progress(self, value)
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::to_value(self, slider_progress)
    }
}

impl<Txt> Callbacks<i32, Txt> for ValidatingIntSliderCallbacks
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
        <ValidatingIntSliderCallbacks as SliderCallbacks<i32, Txt>>::get_widget_creator(
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
        <ValidatingIntSliderCallbacks as IntSliderCallbacks<Txt>>::i32_validate(self, value)
    }
}
