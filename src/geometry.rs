use crate::DEFAULT_DPI;

/// A position in logical coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LogicalPosition<T> {
    pub x: T,
    pub y: T,
}

impl<T> LogicalPosition<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// A position in physical coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PhysicalPosition<T> {
    pub x: T,
    pub y: T,
}

impl<T> PhysicalPosition<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// A size in logical coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LogicalSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> LogicalSize<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// A size in physical coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PhysicalSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> PhysicalSize<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// Converts to a logical position.
pub trait ToLogicalPosition<T> {
    fn to_logical(&self, dpi: T) -> LogicalPosition<T>;
}

/// Converts to a physical position.
pub trait ToPhysicalPosition<T> {
    fn to_physical(&self, dpi: T) -> PhysicalPosition<T>;
}

/// Converts to a logical size.
pub trait ToLogicalSize<T> {
    fn to_logical(&self, dpi: T) -> LogicalSize<T>;
}

/// Converts to a physical size.
pub trait ToPhysicalSize<T> {
    fn to_physical(&self, dpi: T) -> PhysicalSize<T>;
}

impl<T: Clone> ToLogicalPosition<T> for LogicalPosition<T> {
    fn to_logical(&self, _: T) -> LogicalPosition<T> {
        self.clone()
    }
}

impl<T> ToLogicalPosition<T> for PhysicalPosition<T>
where
    T: num::NumCast + std::ops::Mul<T, Output = T> + std::ops::Div<T, Output = T> + Copy,
{
    fn to_logical(&self, dpi: T) -> LogicalPosition<T> {
        LogicalPosition {
            x: self.x * num::cast(DEFAULT_DPI).unwrap() / dpi,
            y: self.y * num::cast(DEFAULT_DPI).unwrap() / dpi,
        }
    }
}

impl<T> ToPhysicalPosition<T> for LogicalPosition<T>
where
    T: num::NumCast + std::ops::Mul<T, Output = T> + std::ops::Div<T, Output = T> + Copy,
{
    fn to_physical(&self, dpi: T) -> PhysicalPosition<T> {
        PhysicalPosition {
            x: self.x * dpi / num::cast(DEFAULT_DPI).unwrap(),
            y: self.y * dpi / num::cast(DEFAULT_DPI).unwrap(),
        }
    }
}

impl<T: Clone> ToPhysicalPosition<T> for PhysicalPosition<T> {
    fn to_physical(&self, _: T) -> PhysicalPosition<T> {
        self.clone()
    }
}

impl<T: Clone> ToLogicalSize<T> for LogicalSize<T> {
    fn to_logical(&self, _: T) -> LogicalSize<T> {
        self.clone()
    }
}

impl<T> ToLogicalSize<T> for PhysicalSize<T>
where
    T: num::NumCast + std::ops::Mul<T, Output = T> + std::ops::Div<T, Output = T> + Copy,
{
    fn to_logical(&self, dpi: T) -> LogicalSize<T> {
        LogicalSize {
            width: self.width * num::cast(DEFAULT_DPI).unwrap() / dpi,
            height: self.height * num::cast(DEFAULT_DPI).unwrap() / dpi,
        }
    }
}

impl<T> ToPhysicalSize<T> for LogicalSize<T>
where
    T: num::NumCast + std::ops::Mul<T, Output = T> + std::ops::Div<T, Output = T> + Copy,
{
    fn to_physical(&self, dpi: T) -> PhysicalSize<T> {
        PhysicalSize {
            width: self.width * dpi / num::cast(DEFAULT_DPI).unwrap(),
            height: self.height * dpi / num::cast(DEFAULT_DPI).unwrap(),
        }
    }
}

impl<T: Clone> ToPhysicalSize<T> for PhysicalSize<T> {
    fn to_physical(&self, _: T) -> PhysicalSize<T> {
        self.clone()
    }
}

/// A position in screen coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}

impl ScreenPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for ScreenPosition {
    fn from(src: (i32, i32)) -> ScreenPosition {
        ScreenPosition::new(src.0, src.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_to_logical_position() {
        let src = LogicalPosition::new(128, 256);
        let dest = src.to_logical(2 * DEFAULT_DPI);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);

        let src = LogicalPosition::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.x - src.x) <= std::f32::EPSILON);
        assert!((dest.y - src.y) <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_physical_position() {
        let src = LogicalPosition::new(128, 256);
        let dest = src.to_physical(2 * DEFAULT_DPI);
        assert!(src.x * 2 == dest.x);
        assert!(src.y * 2 == dest.y);

        let src = LogicalPosition::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.x - src.x * 2.0) <= std::f32::EPSILON);
        assert!((dest.y - src.y * 2.0) <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_physical_position() {
        let src = PhysicalPosition::new(128, 256);
        let dest = src.to_physical(2 * DEFAULT_DPI);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);

        let src = PhysicalPosition::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.x - src.x) <= std::f32::EPSILON);
        assert!((dest.y - src.y) <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_logical_position() {
        let src = PhysicalPosition::new(128.0f32, 256.0f32);
        let dest = src.to_logical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.x - src.x / 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.y - src.y / 2.0).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_logical_size() {
        let src = LogicalSize::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.width - src.width).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_physical_size() {
        let src = LogicalSize::new(128.0f32, 256.0f32);
        let dest = src.to_physical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.width - src.width * 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height * 2.0).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_physical_size() {
        let src = PhysicalSize::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.width - src.width).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_logical_size() {
        let src = PhysicalSize::new(128.0f32, 256.0f32);
        let dest = src.to_logical(2.0f32 * DEFAULT_DPI as f32);
        assert!((dest.width - src.width / 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height / 2.0).abs() <= std::f32::EPSILON);
    }
}
