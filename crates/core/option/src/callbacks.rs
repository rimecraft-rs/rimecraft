use crate::{tooltip_factory::TooltipFactory, SimpleOption};

type ChangeCallback<T> = fn(Option<T>);

type ValueSetter<T> = fn(&mut SimpleOption<T>, value: Option<T>);

type WidgetCreator<T> = fn(&SimpleOption<T>) -> (); // ClickableWidget

trait Callbacks<T> {
    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: ChangeCallback<T>,
    ) -> WidgetCreator<T>;

    fn validate(&self, value: Option<T>) -> Option<T>;
}

trait CyclingCallbacks<T>: Callbacks<T> {
    fn get_values(&self) -> (); // CyclingButtonWidget.Values<T>

    fn value_setter(&self) -> ValueSetter<T> {
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
    ) -> WidgetCreator<T> {
        todo!()
    }
}

trait SliderCallbacks<T>: Callbacks<T> {
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
    ) -> WidgetCreator<T> {
        todo!()
    }
}

trait TypeChangeableCallbacks<T>: CyclingCallbacks<T> + SliderCallbacks<T> {
    fn is_cycling(&self) -> bool;

    fn get_widget_creator(
        &self,
        tooltip_factory: &dyn TooltipFactory<T>,
        game_options: (),
        x: f32,
        y: f32,
        width: f32,
        change_callback: ChangeCallback<T>,
    ) -> WidgetCreator<T> {
        todo!()
    }
}

pub struct PotentialValuesBasedCallbacks<T> {
    values: Vec<T>,
}

impl<T> CyclingCallbacks<T> for PotentialValuesBasedCallbacks<T> {
    fn get_values(&self) -> () {
        todo!()
    }
}

impl<T> Callbacks<T> for PotentialValuesBasedCallbacks<T> {
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
    ) -> WidgetCreator<T> {
        CyclingCallbacks::get_widget_creator(self, tooltip_factory, game_options, x, y, width, change_callback)
    }
}

trait IntSliderCallbacks: SliderCallbacks<i32> {
	fn min_inclusive(&self) -> i32;

	fn max_inclusive(&self) -> i32;
}

pub struct SuppliableIntCallbacks {
	min_boundary: fn() -> i32,
	max_boundary: fn() -> i32,
}
