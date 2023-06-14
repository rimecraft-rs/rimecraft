pub use fastnbt::Tag as NbtType;
pub use fastnbt::Value as NbtElement;

pub use fastnbt::{
    from_bytes, from_bytes_with_opts, from_reader, nbt, to_bytes, to_writer, ByteArray, DeOpts,
    IntArray, LongArray,
};

pub use fastnbt::value::from_value as from_nbt;
pub use fastnbt::value::to_value as to_nbt;

pub use fastsnbt::from_str;

pub type NbtCompound = std::collections::HashMap<String, NbtElement>;
