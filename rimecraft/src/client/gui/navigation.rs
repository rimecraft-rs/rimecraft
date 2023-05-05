#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavigationAxis {
    Horizontal,
    Vertical,
}

impl NavigationAxis {
    pub fn other(&self) -> Self {
        match self {
            NavigationAxis::Horizontal => Self::Vertical,
            NavigationAxis::Vertical => Self::Horizontal,
        }
    }

    pub fn positive_direction(&self) -> NavigationDirection {
        match self {
            NavigationAxis::Horizontal => NavigationDirection::Right,
            NavigationAxis::Vertical => NavigationDirection::Down,
        }
    }

    pub fn negative_direction(&self) -> NavigationDirection {
        match self {
            NavigationAxis::Horizontal => NavigationDirection::Left,
            NavigationAxis::Vertical => NavigationDirection::Up,
        }
    }

    pub fn direction(&self, positive: bool) -> NavigationDirection {
        if positive {
            self.positive_direction()
        } else {
            self.negative_direction()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

impl NavigationDirection {
    pub fn axis(&self) -> NavigationAxis {
        match self {
            NavigationDirection::Up | NavigationDirection::Down => NavigationAxis::Vertical,
            NavigationDirection::Left | NavigationDirection::Right => NavigationAxis::Horizontal,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            NavigationDirection::Up => Self::Down,
            NavigationDirection::Down => Self::Up,
            NavigationDirection::Left => Self::Right,
            NavigationDirection::Right => Self::Left,
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(self, NavigationDirection::Down | NavigationDirection::Right)
    }

    /// Whether the coordinate `a` comes after `b`
    ///
    /// For example, if navigating downwards, `2` comes after `1`, while the opposite is true if navigating upwards.
    /// This always returns `false` if two arguments are equal.
    ///
    /// See [`Self::is_before()`]
    pub fn is_after(&self, a: i32, b: i32) -> bool {
        if self.is_positive() {
            a > b
        } else {
            b > a
        }
    }

    /// Whether the coordinate `a` comes before `b`
    ///
    /// For example, if navigating downwards, `1` comes before `2`, while the opposite is true if navigating upwards.
    /// This always returns `false` if two arguments are equal.
    ///
    /// See [`Self::is_after()`]
    pub fn is_before(&self, a: i32, b: i32) -> bool {
        if self.is_positive() {
            a < b
        } else {
            b < a
        }
    }

    pub fn cmp_coord(&self, a: i32, b: i32) -> i32 {
        if a == b {
            0
        } else if self.is_before(a, b) {
            -1
        } else {
            1
        }
    }
}
