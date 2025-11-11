//! World border types.

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use glam::{DVec3, DVec4};
use math::MathDeltaExt as _;
use maybe::{Maybe, SimpleOwned};
use rimecraft_voxel_math::{BBox, BlockPos};
use serde::{Deserialize, Serialize};
use voxel_shape::VoxelShapeSlice;

const DEFAULT_SIZE: f64 = 5.999997e7;

/// Border of a world.
///
/// See [`Properties`] and [`Self::as_props`] for serialization.
#[derive(Debug)]
pub struct WorldBorder<'a> {
    area: Area,
    max_radius: u32,

    center_x: f64,
    center_z: f64,

    damage_per_block: f64,
    safe_zone: f64,
    warning_time: u32,
    warning_blocks: u32,

    _marker: PhantomData<&'a ()>,
}

/// Mutable variant of [`WorldBorder`].
pub struct WorldBorderMut<'a> {
    immut: WorldBorder<'a>,
    listeners: Vec<Box<dyn Listener + Send + Sync + 'a>>,
}

/// Listener of changes to the world border.
pub trait Listener {
    /// Called when the size of the world border changes.
    fn on_size_change(&mut self, border: &WorldBorder<'_>, size: f64);

    /// Called when the size of the world border is interpolated.
    fn on_interpolate_size(&mut self, border: &WorldBorder<'_>, from: f64, to: f64, time: Duration);

    /// Called when the center of the world border changes.
    fn on_center_changed(&mut self, border: &WorldBorder<'_>, x: f64, z: f64);

    /// Called when the damage per block of the world border changes.
    fn on_damage_per_block_changed(&mut self, border: &WorldBorder<'_>, damage: f64);

    /// Called when the safe zone of the world border changes.
    fn on_safe_zone_changed(&mut self, border: &WorldBorder<'_>, safe_zone: f64);

    /// Called when the warning time of the world border changes.
    fn on_warning_blocks_changed(&mut self, border: &WorldBorder<'_>, warning_blocks: u32);

    /// Called when the warning time of the world border changes.
    fn on_warning_time_changed(&mut self, border: &WorldBorder<'_>, warning_time: u32);
}

/// Key of a [`Listener`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListenerKey(*const ());

unsafe impl Send for ListenerKey {}
unsafe impl Sync for ListenerKey {}

macro_rules! notify {
    ($l:expr => $f:ident($($arg:expr),*$(,)?)) => {
        for listener in &mut $l {
            listener.$f($($arg),*);
        }
    };
}

impl WorldBorder<'_> {
    /// Creates a new world border from default properties.
    pub fn new() -> Self {
        let center_x = 0.0;
        let center_z = 0.0;
        let max_radius = 29999984u32;

        Self {
            area: Area::Static(StaticArea::new(
                DEFAULT_SIZE,
                AreaLocalCx {
                    center_x,
                    center_z,
                    max_radius,
                },
            )),
            max_radius,
            center_x,
            center_z,
            damage_per_block: 0.2,
            safe_zone: 5.0,
            warning_time: 15,
            warning_blocks: 5,
            _marker: PhantomData,
        }
    }

    #[inline]
    const fn area_cx(&self) -> AreaLocalCx {
        AreaLocalCx {
            center_x: self.center_x,
            center_z: self.center_z,
            max_radius: self.max_radius,
        }
    }

    fn contains_raw(&self, x: f64, z: f64, margin: f64) -> bool {
        x >= self.west_bound() - margin
            && x < self.east_bound() + margin
            && z >= self.north_bound() - margin
            && z < self.south_bound() + margin
    }

    /// Checks whether the given object is contained within the world border.
    pub fn contains<Desc>(&self, desc: Desc) -> bool
    where
        Desc: IntoContainsDescriptor,
    {
        desc.descriptors()
            .into_iter()
            .all(|d| self.contains_raw(d.x, d.z, d.margin))
    }

    /// Clamps the given position to the world border.
    pub fn clamp(&self, pos: DVec3) -> DVec3 {
        const F64_TOLERANCE: f64 = 1.0e-5;
        let bounds = self.bounds();
        DVec3::new(
            pos.x.clamp(
                bounds[FourWayDirection::West as usize],
                bounds[FourWayDirection::East as usize] - F64_TOLERANCE,
            ),
            pos.y,
            pos.z.clamp(
                bounds[FourWayDirection::North as usize],
                bounds[FourWayDirection::South as usize] - F64_TOLERANCE,
            ),
        )
    }

    /// Returns the bounds of the world border.
    ///
    /// See [`FourWayDirection`] for indexing from the array.
    #[inline]
    pub fn bounds(&self) -> [f64; 4] {
        self.area.bounds(self.area_cx())
    }

    #[inline]
    fn i_bound_of(&self, direction: FourWayDirection) -> f64 {
        self.area.i_bounds(self.area_cx())[direction as usize]
    }

    /// Returns the bound of the world border in the given direction.
    pub fn bound_of(&self, direction: FourWayDirection) -> f64 {
        self.i_bound_of(direction)
    }

    /// Returns the west bound of the world border.
    pub fn west_bound(&self) -> f64 {
        self.i_bound_of(FourWayDirection::West)
    }

    /// Returns the north bound of the world border.
    pub fn north_bound(&self) -> f64 {
        self.i_bound_of(FourWayDirection::North)
    }

    /// Returns the east bound of the world border.
    pub fn east_bound(&self) -> f64 {
        self.i_bound_of(FourWayDirection::East)
    }

    /// Returns the south bound of the world border.
    pub fn south_bound(&self) -> f64 {
        self.i_bound_of(FourWayDirection::South)
    }

    /// Returns the world border as a voxel shape.
    #[inline]
    pub fn as_voxel_shape(&self) -> Maybe<'_, Arc<VoxelShapeSlice<'static>>> {
        self.area.as_voxel_shape(self.area_cx())
    }

    /// Ticks the world border area.
    #[inline]
    pub fn tick(&mut self) {
        self.area.tick(self.area_cx());
    }

    /// Returns the size of the world border.
    #[inline]
    pub fn size(&self) -> f64 {
        self.area.size()
    }

    /// Returns the center X of the world border.
    #[inline]
    pub fn center_x(&self) -> f64 {
        self.center_x
    }

    /// Returns the center Z of the world border.
    #[inline]
    pub fn center_z(&self) -> f64 {
        self.center_z
    }

    /// Returns the damage increase per block beyond this border in hearts.
    #[inline]
    pub fn damage_per_block(&self) -> f64 {
        self.damage_per_block
    }

    /// Returns the safe zone of the world border.
    #[inline]
    pub fn safe_zone(&self) -> f64 {
        self.safe_zone
    }

    /// Returns the warning time of the world border, in ticks.
    #[inline]
    pub fn warning_time(&self) -> u32 {
        self.warning_time
    }

    /// Returns the warning blocks of the world border.
    #[inline]
    pub fn warning_blocks(&self) -> u32 {
        self.warning_blocks
    }

    /// Returns the maximum radius of the world border in blocks.
    #[inline]
    pub fn max_radius(&self) -> u32 {
        self.max_radius
    }

    /// Returns the properties of the world border.
    pub fn as_props(&self) -> Properties {
        Properties {
            center_x: self.center_x,
            center_z: self.center_z,
            damage_per_block: self.damage_per_block,
            safe_zone: self.safe_zone,
            warning_blocks: self.warning_blocks,
            warning_time: self.warning_time,
            size: self.size(),
            size_lerp_time: self.area.size_lerp_time().as_millis() as u64,
            size_lerp_target: self.area.size_lerp_target(),
        }
    }
}

impl<'a> WorldBorderMut<'a> {
    /// Creates a new mutable world border from an immutable one.
    pub fn new(border: WorldBorder<'a>) -> Self {
        Self {
            immut: border,
            listeners: Vec::new(),
        }
    }

    /// Adds a listener to the world border changes.
    pub fn add_listener<L>(&mut self, listener: L) -> ListenerKey
    where
        L: Listener + Send + Sync + 'a,
    {
        let listener = Box::new(listener);
        let key = ListenerKey(std::ptr::from_ref::<L>(&*listener).cast());
        self.listeners.push(listener);
        key
    }

    /// Removes a listener from the world border.
    pub fn remove_listener(
        &mut self,
        key: ListenerKey,
    ) -> Option<Box<dyn Listener + Send + Sync + 'a>> {
        self.listeners
            .iter()
            .position(|l| std::ptr::eq(std::ptr::from_ref(&**l).cast(), key.0))
            .map(|i| self.listeners.swap_remove(i))
    }

    /// Interpolates the size of the world border, and notifies listeners.
    pub fn interpolate_size(&mut self, from: f64, to: f64, time: Duration) {
        self.immut.area = if approx::abs_diff_eq!(from, to) {
            Area::Static(StaticArea::new(to, self.area_cx()))
        } else {
            Area::Moving(MovingArea::new(from, to, time))
        };
        notify!(self.listeners => on_interpolate_size(&self.immut, from, to, time));
    }

    /// Sets the size of the world border, and notifies listeners.
    ///
    /// See [`Self::interpolate_size`] for transitioning size linearly by time.
    pub fn set_size(&mut self, size: f64) {
        self.immut.area = Area::Static(StaticArea::new(size, self.area_cx()));
        notify!(self.listeners => on_size_change(&self.immut, size));
    }

    /// Sets the center of the world border, and notifies listeners.
    pub fn set_center(&mut self, x: f64, z: f64) {
        self.immut.center_x = x;
        self.immut.center_z = z;
        self.immut.area.on_center_changed(self.area_cx());
        notify!(self.listeners => on_center_changed(&self.immut, x, z));
    }

    /// Sets the damage per block of the world border, and notifies listeners.
    pub fn set_damage_per_block(&mut self, damage_per_block: f64) {
        self.immut.damage_per_block = damage_per_block;
        notify!(self.listeners => on_damage_per_block_changed(&self.immut, damage_per_block));
    }

    /// Sets the safe zone of the world border, and notifies listeners.
    pub fn set_safe_zone(&mut self, safe_zone: f64) {
        self.immut.safe_zone = safe_zone;
        notify!(self.listeners => on_safe_zone_changed(&self.immut, safe_zone));
    }

    /// Sets the warning time of the world border, and notifies listeners.
    pub fn set_warning_blocks(&mut self, warning_blocks: u32) {
        self.immut.warning_blocks = warning_blocks;
        notify!(self.listeners => on_warning_blocks_changed(&self.immut, warning_blocks))
    }

    /// Sets the warning time of the world border, and notifies listeners.
    pub fn set_warning_time(&mut self, warning_time: u32) {
        self.immut.warning_time = warning_time;
        notify!(self.listeners => on_warning_time_changed(&self.immut, warning_time));
    }

    /// Loads the world border from the given properties.
    pub fn load_from_props(&mut self, props: Properties) {
        self.set_center(props.center_x, props.center_z);
        self.set_damage_per_block(props.damage_per_block);
        self.set_safe_zone(props.safe_zone);
        self.set_warning_blocks(props.warning_blocks);
        self.set_warning_time(props.warning_time);
        if props.size_lerp_time > 0 {
            self.interpolate_size(
                props.size,
                props.size_lerp_target,
                Duration::from_millis(props.size_lerp_time),
            );
        } else {
            self.set_size(props.size);
        }
    }
}

impl<'a> Deref for WorldBorderMut<'a> {
    type Target = WorldBorder<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.immut
    }
}

/// Descriptor for checking whether a point is contained within the world border.
#[derive(Debug, Clone, Copy)]
pub struct ContainsDescriptor {
    /// X coordinate.
    pub x: f64,
    /// Z coordinate.
    pub z: f64,
    /// Additive margin from the four-way bounds.
    pub margin: f64,
}

/// Types which can be converted into a set of [`ContainsDescriptor`]s.
pub trait IntoContainsDescriptor {
    /// Converts into a set of [`ContainsDescriptor`]s.
    fn descriptors(self) -> impl IntoIterator<Item = ContainsDescriptor>;
}

impl Default for WorldBorder<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
struct AreaLocalCx {
    center_x: f64,
    center_z: f64,
    max_radius: u32,
}

#[derive(Debug)]
enum Area {
    Static(StaticArea),
    Moving(MovingArea),
}

impl Area {
    fn bounds(&self, cx: AreaLocalCx) -> [f64; 4] {
        self.i_bounds(cx)
    }

    #[inline]
    fn i_bounds(&self, cx: AreaLocalCx) -> [f64; 4] {
        match self {
            Self::Static(area) => area.bounds,
            Self::Moving(_) => {
                let radius = self.i_size() / 2.0;
                let max_radius = cx.max_radius as f64;
                DVec4::new(
                    cx.center_x - radius,
                    cx.center_z - radius,
                    cx.center_x + radius,
                    cx.center_z + radius,
                )
                .clamp(DVec4::splat(-max_radius), DVec4::splat(max_radius))
                .into()
            }
        }
    }

    fn size(&self) -> f64 {
        self.i_size()
    }

    #[inline]
    fn i_size(&self) -> f64 {
        match self {
            Self::Static(area) => area.size,
            Self::Moving(area) => {
                let d = area.start.elapsed().div_duration_f64(area.duration);
                if d < 1.0 {
                    area.old_size.lerp(area.new_size, d)
                } else {
                    area.new_size
                }
            }
        }
    }

    #[inline]
    fn size_lerp_time(&self) -> Duration {
        match self {
            Self::Static(_) => Duration::ZERO,
            Self::Moving(area) => area.end.saturating_duration_since(Instant::now()),
        }
    }

    #[inline]
    fn size_lerp_target(&self) -> f64 {
        match self {
            Self::Static(area) => area.size,
            Self::Moving(area) => area.new_size,
        }
    }

    #[inline]
    fn on_center_changed(&mut self, cx: AreaLocalCx) {
        if let Self::Static(area) = self {
            area.calculate_bounds(cx);
        }
    }

    fn as_voxel_shape(&self, cx: AreaLocalCx) -> Maybe<'_, Arc<VoxelShapeSlice<'static>>> {
        match self {
            Self::Static(area) => Maybe::Borrowed(&area.shape),
            Self::Moving(_) => {
                let bounds = self.bounds(cx);
                Maybe::Owned(SimpleOwned(
                    voxel_shape::combine_with(
                        voxel_shape::unbounded(),
                        &voxel_shape::cuboid(BBox::from_raw(
                            DVec3::new(
                                bounds[FourWayDirection::West as usize].floor(),
                                f64::NEG_INFINITY,
                                bounds[FourWayDirection::North as usize].floor(),
                            ),
                            DVec3::new(
                                bounds[FourWayDirection::East as usize].ceil(),
                                f64::INFINITY,
                                bounds[FourWayDirection::South as usize].ceil(),
                            ),
                        )),
                        |a, b| a && !b,
                    )
                    .simplify(),
                ))
            }
        }
    }

    fn tick(&mut self, cx: AreaLocalCx) {
        if let Self::Moving(area) = self
            && area.end <= Instant::now()
        {
            *self = Self::Static(StaticArea::new(area.new_size, cx))
        }
    }
}

/// Four-way direction representing sides of a border.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
#[allow(clippy::exhaustive_enums)]
pub enum FourWayDirection {
    /// West.
    West = 0,
    /// North.
    North = 1,
    /// East.
    East = 2,
    /// South.
    South = 3,
}

#[derive(Debug)]
struct MovingArea {
    start: Instant,
    end: Instant,
    duration: Duration,

    old_size: f64,
    new_size: f64,
}

impl MovingArea {
    fn new(old_size: f64, new_size: f64, duration: Duration) -> Self {
        let start = Instant::now();
        Self {
            start,
            end: start + duration,
            duration,
            old_size,
            new_size,
        }
    }
}

#[derive(Debug)]
struct StaticArea {
    size: f64,
    bounds: [f64; 4],
    shape: Arc<VoxelShapeSlice<'static>>,
}

impl StaticArea {
    fn new(size: f64, cx: AreaLocalCx) -> Self {
        let mut this = Self {
            size,
            bounds: [0.0; 4],
            shape: voxel_shape::empty().clone(),
        };
        this.calculate_bounds(cx);
        this
    }

    fn calculate_bounds(&mut self, cx: AreaLocalCx) {
        let radius = self.size / 2.0;
        let max_radius = cx.max_radius as f64;
        self.bounds = DVec4::new(
            cx.center_x - radius,
            cx.center_z - radius,
            cx.center_x + radius,
            cx.center_z + radius,
        )
        .clamp(DVec4::splat(-max_radius), DVec4::splat(max_radius))
        .into();
        self.shape = voxel_shape::combine_with(
            voxel_shape::unbounded(),
            &voxel_shape::cuboid(BBox::from_raw(
                DVec3::new(
                    self.bounds[FourWayDirection::West as usize].floor(),
                    f64::NEG_INFINITY,
                    self.bounds[FourWayDirection::North as usize].floor(),
                ),
                DVec3::new(
                    self.bounds[FourWayDirection::East as usize].ceil(),
                    f64::INFINITY,
                    self.bounds[FourWayDirection::South as usize].ceil(),
                ),
            )),
            |a, b| a && !b,
        )
        .simplify();
    }
}

impl IntoContainsDescriptor for ContainsDescriptor {
    #[inline]
    fn descriptors(self) -> impl IntoIterator<Item = Self> {
        std::iter::once(self)
    }
}

impl IntoContainsDescriptor for (f64, f64) {
    #[inline]
    fn descriptors(self) -> impl IntoIterator<Item = ContainsDescriptor> {
        ContainsDescriptor {
            x: self.0,
            z: self.1,
            margin: 0.0,
        }
        .descriptors()
    }
}

impl IntoContainsDescriptor for BlockPos {
    #[inline]
    fn descriptors(self) -> impl IntoIterator<Item = ContainsDescriptor> {
        (self.x() as f64, self.z() as f64).descriptors()
    }
}

impl Debug for WorldBorderMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WorldBorderMut")
            .field(&self.immut)
            .finish_non_exhaustive()
    }
}

/// Represents bare, serializable properties of a [`WorldBorder`].
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct Properties {
    pub center_x: f64,
    pub center_z: f64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: u32,
    pub warning_time: u32,
    pub size: f64,
    pub size_lerp_time: u64,
    pub size_lerp_target: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PropertiesSerialized {
    border_center_x: f64,
    border_center_z: f64,
    border_size: f64,
    border_size_lerp_time: u64,
    border_size_lerp_target: f64,
    border_safe_zone: f64,
    border_damage_per_block: f64,
    border_warning_blocks: u32,
    border_warning_time: u32,
}

impl Serialize for Properties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized = PropertiesSerialized {
            border_center_x: self.center_x,
            border_center_z: self.center_z,
            border_size: self.size,
            border_size_lerp_time: self.size_lerp_time,
            border_size_lerp_target: self.size_lerp_target,
            border_safe_zone: self.safe_zone,
            border_damage_per_block: self.damage_per_block,
            border_warning_blocks: self.warning_blocks,
            border_warning_time: self.warning_time,
        };
        serialized.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Properties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serialized = PropertiesSerialized::deserialize(deserializer)?;
        const BOUND: f64 = 9999984e7;
        Ok(Self {
            center_x: serialized.border_center_x.clamp(-BOUND, BOUND),
            center_z: serialized.border_center_z.clamp(-BOUND, BOUND),
            size: serialized.border_size,
            size_lerp_time: serialized.border_size_lerp_time,
            size_lerp_target: serialized.border_size_lerp_target,
            safe_zone: serialized.border_safe_zone,
            damage_per_block: serialized.border_damage_per_block,
            warning_blocks: serialized.border_warning_blocks,
            warning_time: serialized.border_warning_time,
        })
    }
}
