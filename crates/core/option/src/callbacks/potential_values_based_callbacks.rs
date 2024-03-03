use super::*;

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
        change_callback: &ChangeCallback<T>,
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