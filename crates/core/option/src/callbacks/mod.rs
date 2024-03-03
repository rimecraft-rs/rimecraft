pub mod potential_values_based_callbacks;
pub mod suppliable_int_callbacks;
pub mod validating_int_slider_callbacks;

use rimecraft_text::Texts;

use crate::{tooltip_factory::TooltipFactory, SimpleOption};

type ChangeCallback<T> = dyn Fn(Option<T>);

type ValueSetter<T, Txt>
where
    Txt: Texts,
= fn(&mut SimpleOption<T, Txt>, value: Option<T>);

type WidgetCreator<T, Txt>
where
    Txt: Texts,
= fn(&SimpleOption<T, Txt>) -> (); // ClickableWidget

trait Callbacks<T, Txt>
where
    Txt: Texts,
{
    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: &ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt>;

    fn validate(&self, value: Option<T>) -> Option<T>;
}

trait CyclingCallbacks<T, Txt>: Callbacks<T, Txt>
where
    Txt: Texts,
{
    fn get_values(&self) -> (); // CyclingButtonWidget.Values<T>

    fn value_setter(&self) -> ValueSetter<T, Txt> {
        |option, value| option.set_value(value)
    }

    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: &ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt> {
        todo!()
    }
}

trait SliderCallbacks<T, Txt>: Callbacks<T, Txt>
where
    Txt: Texts,
{
    fn to_slider_progress(&self, value: T) -> f32;

    fn to_value(&self, slider_progress: f32) -> T;

    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: &ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt> {
        todo!()
    }
}

trait TypeChangeableCallbacks<T, Txt>: CyclingCallbacks<T, Txt> + SliderCallbacks<T, Txt>
where
    Txt: Texts,
{
    fn is_cycling(&self) -> bool;

    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: &ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt> {
        todo!()
    }
}

trait IntSliderCallbacks<Txt>: SliderCallbacks<i32, Txt>
where
    Txt: Texts,
{
    fn min_inclusive(&self) -> i32;

    fn max_inclusive(&self) -> i32;

    fn to_slider_progress(&self, value: i32) -> f32 {
        rimecraft_math::_map(
            value as f32,
            self.min_inclusive() as f32,
            self.max_inclusive() as f32,
            0.0,
            1.0,
        )
    }

    fn to_value(&self, slider_progress: f32) -> i32 {
        rimecraft_math::_map(
            slider_progress,
            0.0,
            1.0,
            self.min_inclusive() as f32,
            self.max_inclusive() as f32,
        )
        .floor() as i32
    }

    fn i32_validate(&self, value: Option<i32>) -> Option<i32>;

    fn with_modifier<R, IR, RI, F, ToP, ToV>(
        &self,
        progress_to_value: IR,
        value_to_progress: RI,
    ) -> impl SliderCallbacks<R, Txt>
    where
        IR: Fn(Option<i32>) -> Option<R>,
        RI: Fn(Option<R>) -> Option<i32>,
        F: Fn(Option<i32>) -> Option<i32>,
        ToP: Fn(i32) -> f32,
        ToV: Fn(f32) -> i32,
    {
        struct Impl<IR, RI, F, ToP, ToV> {
            progress_to_value: IR,
            value_to_progress: RI,
            i32_validate: F,
            to_slider_progress: ToP,
            to_value: ToV,
        }

        impl<R, IR, RI, F, ToP, ToV, Txt> SliderCallbacks<R, Txt> for Impl<IR, RI, F, ToP, ToV>
        where
            Txt: Texts,
            IR: Fn(Option<i32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<i32>,
            F: Fn(Option<i32>) -> Option<i32>,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
        {
            fn to_slider_progress(&self, value: R) -> f32 {
                let progress = (self.value_to_progress)(Some(value)).unwrap();
                (self.to_slider_progress)(progress)
            }

            fn to_value(&self, slider_progress: f32) -> R {
                let value = (self.to_value)(slider_progress);
                (self.progress_to_value)(Some(value)).unwrap()
            }
        }

        impl<R, IR, RI, F, ToP, ToV, Txt> Callbacks<R, Txt> for Impl<IR, RI, F, ToP, ToV>
        where
            Txt: Texts,
            IR: Fn(Option<i32>) -> Option<R>,
            RI: Fn(Option<R>) -> Option<i32>,
            F: Fn(Option<i32>) -> Option<i32>,
            ToP: Fn(i32) -> f32,
            ToV: Fn(f32) -> i32,
        {
            fn validate(&self, value: Option<R>) -> Option<R> {
                let i = (self.value_to_progress)(value);
                let invalidated = (self.i32_validate)(i);
                (self.progress_to_value)(invalidated)
            }

            fn get_widget_creator(
                &self,
                tooltip_factory: &dyn TooltipFactory<R>,
                game_options: (),
                x: f32,
                y: f32,
                width: f32,
                change_callback: &ChangeCallback<R>,
            ) -> WidgetCreator<R, Txt> {
                SliderCallbacks::get_widget_creator(
                    self,
                    tooltip_factory,
                    game_options,
                    x,
                    y,
                    width,
                    change_callback,
                )
            }
        }

        Impl {
            value_to_progress,
            progress_to_value,
            i32_validate: |value| self.i32_validate(value),
            to_slider_progress: |value| IntSliderCallbacks::to_slider_progress(self, value),
            to_value: |value| IntSliderCallbacks::to_value(self, value),
        }
    }
}
