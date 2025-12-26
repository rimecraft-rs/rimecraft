use rcutil::Static;
use remap::{remap, remap_method};
use rimecraft_voxel_math::{BlockPos, HitResult, direction::Direction, glam::DVec3};

/// Hit result with type of blocks.
#[derive(Debug, Clone)]
#[remap(yarn = "BlockHitResult", mojmaps = "BlockHitResult")]
pub struct BlockHitResult {
    pos: DVec3,
    side: Direction,
    block_pos: BlockPos,
    missed: bool,
    inside_block: bool,
    against_world_border: bool,
}

impl BlockHitResult {
    /// Creates a new block hit result.
    #[inline]
    pub const fn new(pos: DVec3, side: Direction, block_pos: BlockPos, inside_block: bool) -> Self {
        Self {
            pos,
            side,
            block_pos,
            missed: false,
            inside_block,
            against_world_border: false,
        }
    }

    /// Creates a new missed hit result.
    #[inline]
    #[remap_method(yarn = "createMissed", mojmaps = "miss")]
    pub const fn missed(pos: DVec3, side: Direction, block_pos: BlockPos) -> Self {
        Self {
            missed: true,
            ..Self::new(pos, side, block_pos, false)
        }
    }

    /// Makes this hit result be marked as against world border.
    #[inline]
    #[remap_method(yarn = "againstWorldBorder", mojmaps = "hitBorder")]
    pub fn against_world_border(self) -> Self {
        Self {
            against_world_border: true,
            ..self
        }
    }

    /// Makes this hit result face a new given side.
    #[inline]
    #[remap_method(yarn = "withSide", mojmaps = "withDirection")]
    pub fn with_side(self, side: Direction) -> Self {
        Self { side, ..self }
    }

    /// Makes this hit result locate at given block position.
    #[inline]
    #[remap_method(yarn = "withBlockPos", mojmaps = "withPosition")]
    pub fn with_block_pos(self, pos: BlockPos) -> Self {
        Self {
            block_pos: pos,
            ..self
        }
    }

    /// Returns block positoin of this hit result.
    #[inline]
    #[remap_method(yarn = "getBlockPos", mojmaps = "getBlockPos")]
    pub fn block_pos(&self) -> BlockPos {
        self.block_pos
    }

    /// Returns side of this hit result.
    #[inline]
    #[remap_method(yarn = "getSide", mojmaps = "getDirection")]
    pub fn side(&self) -> Direction {
        self.side
    }

    /// Whether the hit ends inside the block.
    #[inline]
    #[remap_method(yarn = "isInsideBlock", mojmaps = "isInside")]
    pub fn is_inside_block(&self) -> bool {
        self.inside_block
    }

    /// Whether the hit ends against the world border.
    #[inline]
    #[remap_method(yarn = "isAgainstWorldBorder", mojmaps = "isWorldBorderHit")]
    pub fn is_against_world_border(&self) -> bool {
        self.against_world_border
    }
}

impl HitResult for BlockHitResult {
    #[inline]
    fn pos(&self) -> DVec3 {
        self.pos
    }

    #[inline]
    fn is_missed(&self) -> bool {
        self.missed
    }
}

impl Static for BlockHitResult {}
