pub trait TooltipFactory<T> {
	fn apply(&self, value: T) -> Option<()>; // Option<Tooltip>
}