use std::{ops::Deref, sync::atomic::AtomicU16};

use crate::{prelude::*, util::math::ChunkPos};

use super::HeightLimitView;

pub struct RawChunk<W: HeightLimitView> {
    pub pos: ChunkPos,
    pub height_limit_view: std::sync::Arc<W>,
    pub upgrade_data: UpgradeData,
}

impl<W: HeightLimitView> RawChunk<W> {}

pub struct ChunkSection {
    non_empty_block_count: AtomicU16,
    random_tickable_block_count: AtomicU16,
    non_empty_fluid_count: AtomicU16,
}

pub struct UpgradeData {
    sides_to_upgrade: Vec<crate::util::math::EightWayDirection>,
    block_ticks: Vec<super::tick::Tick<crate::block::Block>>,
    fluid_ticks: Vec<super::tick::Tick<crate::fluid::Fluid>>,
    /// The `0` (usize) field is for the outer vector's lenth.
    center_indices_to_upgrade: (usize, Vec<Vec<i32>>),
}

impl UpgradeData {
    const INDICES_KEY: &str = "Indices";

    pub fn new(nbt: &crate::nbt::NbtCompound, world: &impl HeightLimitView) -> Self {
        let mut s = Self {
            sides_to_upgrade: Vec::new(),
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            center_indices_to_upgrade: (world.count_vertical_sections() as usize, Vec::new()),
        };

        if let Some(compound) = nbt.get_compound(Self::INDICES_KEY) {
            for i in 0..s.center_indices_to_upgrade.0 {
                let string = i.to_string();
                s.center_indices_to_upgrade.1.push(
                    if let Some(slice) = compound.get_i32_slice(&string) {
                        slice.to_vec()
                    } else {
                        Vec::new()
                    },
                )
            }
        }

        let j = nbt.get_i32("Sides").unwrap_or_default();

        for ewd in crate::util::math::EightWayDirection::values() {
            if (j & 1 << ewd as u8) != 0 {
                s.sides_to_upgrade.push(ewd);
            }
        }

        Self::add_neighbor_ticks(
            nbt,
            "neighbor_block_ticks",
            |id| {
                Some(
                    crate::registry::BLOCK
                        .get_from_id(&Identifier::try_parse(id).ok()?)
                        .map(|e| e.1.deref().clone())
                        .unwrap_or_default(),
                )
            },
            &mut s.block_ticks,
        );

        Self::add_neighbor_ticks(
            nbt,
            "neighbor_fluid_ticks",
            |id| {
                Some(
                    crate::registry::FLUID
                        .get_from_id(&Identifier::try_parse(id).ok()?)
                        .map(|e| e.1.deref().clone())
                        .unwrap_or_default(),
                )
            },
            &mut s.fluid_ticks,
        );

        s
    }

    fn add_neighbor_ticks<T, F>(
        nbt: &crate::nbt::NbtCompound,
        key: &str,
        name_to_type: F,
        ticks: &mut Vec<super::tick::Tick<T>>,
    ) where
        F: Fn(&str) -> Option<T>,
    {
        if let Some(list) = nbt.get_slice(key) {
            for value in list.iter() {
                if let Some(tick) = super::tick::Tick::from_nbt(
                    match value {
                        crate::nbt::NbtElement::Compound(c) => c,
                        _ => continue,
                    },
                    |n| name_to_type(n),
                ) {
                    ticks.push(tick)
                }
            }
        }
    }
}

pub mod palette {
    use std::hash::Hash;

    /// A palette maps objects from and to small integer IDs that uses less
    /// number of bits to make storage smaller.
    ///
    /// While the objects palettes handle are already represented by integer
    /// IDs, shrinking IDs in cases where only a few appear can further reduce
    /// storage space and network traffic volume.
    pub trait Palette<T>: Clone {
        /// The id of an object is this palette.
        ///
        /// If the object does not yet exist in this palette, this palette will
        /// register the object. If the palette is too small to include this object,
        /// a [`ResizeListen`] will be called and
        /// this palette may be discarded.
        fn get_index(&self, value: &T) -> u32;

        /// True if any entry in this palette passes the `predicate`.
        fn any<P>(&self, p: P) -> bool
        where
            P: Fn(&T) -> bool;

        /// The object associated with the given `id`.
        fn get(&self, index: u32) -> Option<&T>;

        /// The serialized lenth of this palette in a byte buf, in bytes.
        fn packet_len(&self) -> usize;

        fn create(
            bits: usize,
            index: &'static impl crate::collections::Indexed<T>,
            listener: &impl ResizeListen<T>,
            var4: &[T],
        ) -> Self;

        fn read_buf<B>(&mut self, buf: &mut B)
        where
            B: bytes::Buf;

        fn write_buf<B>(&self, buf: &mut B)
        where
            B: bytes::BufMut;
    }

    /// Listan for palette that requires more bits to hold a newly indexed
    /// object. A no-op listener may be used if the palette does not have to
    /// resize.
    pub trait ResizeListen<T> {
        /// Callback for a palette's request to resize to at least `new_bits`
        /// for each entry and to update the storage correspondingly in order to
        /// accommodate the new object. After the resize is completed in this method,
        /// returns the ID assigned to the `object` in the updated palette.
        ///
        /// Return the ID for the `object` in the (possibly new) palette.
        fn on_resize(&self, new_bits: i32, object: &T) -> i32;
    }

    pub struct Paletted<T: 'static, P, S>
    where
        P: Palette<T>,
        S: crate::util::collections::PaletteStoragge,
    {
        index: crate::util::StaticRef<dyn crate::util::collections::Indexed<T>>,
        data: ContainerData<T, P, S>,
    }

    struct ContainerData<T, P, S>
    where
        P: Palette<T>,
        S: crate::util::collections::PaletteStoragge,
    {
        bits: usize,
        palette: P,
        storage: S,
        _t: std::marker::PhantomData<T>,
    }

    impl<T> Palette<T> for crate::util::StaticRef<dyn crate::collections::Indexed<T>> {
        fn get_index(&self, value: &T) -> u32 {
            self.get_raw_id(value).unwrap_or_default() as u32
        }

        fn any<P>(&self, _p: P) -> bool
        where
            P: Fn(&T) -> bool,
        {
            true
        }

        fn get(&self, index: u32) -> Option<&T> {
            crate::collections::Indexed::get(self.0, index as usize)
        }

        fn packet_len(&self) -> usize {
            crate::util::VarInt(0).len()
        }

        fn create(
            _bits: usize,
            index: &'static impl crate::collections::Indexed<T>,
            _listener: &impl ResizeListen<T>,
            _var4: &[T],
        ) -> Self {
            Self(index)
        }

        fn read_buf<B>(&mut self, _buf: &mut B)
        where
            B: bytes::Buf,
        {
        }

        fn write_buf<B>(&self, _buf: &mut B)
        where
            B: bytes::BufMut,
        {
        }
    }
}
