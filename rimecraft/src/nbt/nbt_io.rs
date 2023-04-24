use std::io::{self, Read, Write};

use crate::util::read::ReadHelper;

use super::{NbtElement, NbtTagSizeTracker, NbtType};

pub fn write(nbt: &NbtElement, output: &mut impl Write) -> io::Result<()> {
    output.write(&[nbt.get_type()])?;
    if nbt.get_type() == 0 {
        return Ok(());
    }
    output.write(&0_u16.to_be_bytes())?;
    nbt.write(output)
}

pub fn read(
    input: &mut impl Read,
    depth: Option<usize>,
    tracker: &mut NbtTagSizeTracker,
) -> io::Result<NbtElement> {
    let mut reader = ReadHelper::new(input);
    let i = depth.unwrap_or(0);
    let b = reader.read_u8()?;
    if b == 0 {
        Ok(NbtElement::End)
    } else {
        NbtType::from_id(b).unwrap().read(input, i, tracker)
    }
}
