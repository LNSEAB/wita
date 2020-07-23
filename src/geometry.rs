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
    fn to_logical(&self, scale: T) -> LogicalPosition<T>;
}

/// Converts to a physical position.
pub trait ToPhysicalPosition<T> {
    fn to_physical(&self, scale: T) -> PhysicalPosition<T>;
}

/// Converts to a logical size.
pub trait ToLogicalSize<T> {
    fn to_logical(&self, scale: T) -> LogicalSize<T>;
}

/// Converts to a physical size.
pub trait ToPhysicalSize<T> {
    fn to_physical(&self, scale: T) -> PhysicalSize<T>;
}

impl<T: Clone> ToLogicalPosition<T> for LogicalPosition<T> {
    fn to_logical(&self, _: T) -> LogicalPosition<T> {
        self.clone()
    }
}

impl<T> ToLogicalPosition<T> for PhysicalPosition<T>
where
    T: std::ops::Div<T, Output = T> + Copy,
{
    fn to_logical(&self, scale: T) -> LogicalPosition<T> {
        LogicalPosition {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
}

impl<T> ToPhysicalPosition<T> for LogicalPosition<T>
where
    T: std::ops::Mul<T, Output = T> + Copy,
{
    fn to_physical(&self, scale: T) -> PhysicalPosition<T> {
        PhysicalPosition {
            x: self.x * scale,
            y: self.y * scale,
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
    T: std::ops::Div<T, Output = T> + Copy,
{
    fn to_logical(&self, scale: T) -> LogicalSize<T> {
        LogicalSize {
            width: self.width / scale,
            height: self.height / scale,
        }
    }
}

impl<T> ToPhysicalSize<T> for LogicalSize<T>
where
    T: std::ops::Mul<T, Output = T> + Copy,
{
    fn to_physical(&self, scale: T) -> PhysicalSize<T> {
        PhysicalSize {
            width: self.width * scale,
            height: self.height * scale,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_to_logical_position() {
        let src = LogicalPosition::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32);
        assert!((dest.x - src.x).abs() <= std::f32::EPSILON);
        assert!((dest.y - src.y).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_physical_position() {
        let src = LogicalPosition::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32);
        assert!((dest.x - src.x * 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.y - src.y * 2.0).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_physical_position() {
        let src = PhysicalPosition::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32);
        assert!((dest.x - src.x).abs() <= std::f32::EPSILON);
        assert!((dest.y - src.y).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_logical_position() {
        let src = PhysicalPosition::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32);
        assert!((dest.x - src.x / 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.y - src.y / 2.0).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_logical_size() {
        let src = LogicalSize::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32);
        assert!((dest.width - src.width).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn logical_to_physical_size() {
        let src = LogicalSize::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32);
        assert!((dest.width - src.width * 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height * 2.0).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_physical_size() {
        let src = PhysicalSize::new(128.0, 256.0);
        let dest = src.to_physical(2.0f32);
        assert!((dest.width - src.width).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height).abs() <= std::f32::EPSILON);
    }

    #[test]
    fn physical_to_logical_size() {
        let src = PhysicalSize::new(128.0, 256.0);
        let dest = src.to_logical(2.0f32);
        assert!((dest.width - src.width / 2.0).abs() <= std::f32::EPSILON);
        assert!((dest.height - src.height / 2.0).abs() <= std::f32::EPSILON);
    }
}
