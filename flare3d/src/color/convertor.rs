pub fn strip_prefix(color: &str) -> &str {
	color.strip_prefix("0x").unwrap_or(color.strip_prefix("#").unwrap_or(color))
}

pub fn rgb_to_hex(color: usize) -> String {
	format!("0x{:0>6X}", color)
}

pub fn hex_to_rgb(color: &str) -> usize {
	usize::from_str_radix(strip_prefix(color), 16).unwrap()
}