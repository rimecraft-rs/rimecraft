use rimecraft_text::Texts;

use crate::{tooltip_factory::TooltipFactory, SimpleOption};

type ChangeCallback<T> = fn(Option<T>);

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
        change_callback: ChangeCallback<T>,
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
        change_callback: ChangeCallback<T>,
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
        change_callback: ChangeCallback<T>,
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
        change_callback: ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt> {
        todo!()
    }
}

pub struct PotentialValuesBasedCallbacks<T> {
    values: Vec<T>,
}

impl<T, Txt> CyclingCallbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: Texts,
{
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<T, Txt> Callbacks<T, Txt> for PotentialValuesBasedCallbacks<T>
where
    Txt: Texts,
{
    fn validate(&self, value: Option<T>) -> Option<T> {
        todo!()
    }

    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: ChangeCallback<T>,
    ) -> WidgetCreator<T, Txt> {
        CyclingCallbacks::get_widget_creator(
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

trait IntSliderCallbacks<Txt>: SliderCallbacks<i32, Txt>
where
    Txt: Texts,
{
    fn min_inclusive(&self) -> i32;

    fn max_inclusive(&self) -> i32;
}

pub struct SuppliableIntCallbacks {
    min_boundary: fn() -> i32,
    max_boundary: fn() -> i32,
}
