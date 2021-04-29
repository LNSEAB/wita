use crate::DEFAULT_DPI;

/// A generic position
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Position<T, U> {
    pub x: T,
    pub y: T,
    #[cfg_attr(feature = "serde", serde(skip))]
    _u: std::marker::PhantomData<U>,
}

impl<T, U> Position<T, U> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            _u: std::marker::PhantomData,
        }
    }
}

impl<T, U> Position<T, U>
where
    T: num::NumCast,
{
    #[inline]
    pub fn cast<R>(self) -> Position<R, U>
    where
        R: num::NumCast,
    {
        Position::new(num::cast(self.x).unwrap(), num::cast(self.y).unwrap())
    }
}

impl<T, U> From<[T; 2]> for Position<T, U>
where
    T: Copy,
{
    #[inline]
    fn from(src: [T; 2]) -> Position<T, U> {
        Position::new(src[0], src[1])
    }
}

impl<T, U> From<(T, T)> for Position<T, U> {
    #[inline]
    fn from(src: (T, T)) -> Position<T, U> {
        Position::new(src.0, src.1)
    }
}

/// A generic size
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Size<T, U> {
    pub width: T,
    pub height: T,
    #[cfg_attr(feature = "serde", serde(skip))]
    _u: std::marker::PhantomData<U>,
}

impl<T, U> Size<T, U> {
    #[inline]
    pub fn new(width: T, height: T) -> Self {
        Self {
            width,
            height,
            _u: std::marker::PhantomData,
        }
    }
}

impl<T, U> Size<T, U>
where
    T: num::NumCast,
{
    #[inline]
    pub fn cast<R>(self) -> Size<R, U>
    where
        R: num::NumCast,
    {
        Size::new(
            num::cast(self.width).unwrap(),
            num::cast(self.height).unwrap(),
        )
    }
}

impl<T, U> From<[T; 2]> for Size<T, U>
where
    T: Copy,
{
    #[inline]
    fn from(src: [T; 2]) -> Size<T, U> {
        Size::new(src[0], src[1])
    }
}

impl<T, U> From<(T, T)> for Size<T, U> {
    #[inline]
    fn from(src: (T, T)) -> Size<T, U> {
        Size::new(src.0, src.1)
    }
}

/// Logical coordinate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Logical;
/// Physical coordinate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Physical;
/// Screen coordinate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Screen;

/// A position in logical coordinate.
pub type LogicalPosition<T> = Position<T, Logical>;
/// A size in logical coordinate.
pub type LogicalSize<T> = Size<T, Logical>;
/// A position in physical coordinate.
pub type PhysicalPosition<T> = Position<T, Physical>;
/// A size in physical coordinate.
pub type PhysicalSize<T> = Size<T, Physical>;
/// A position in screen coordinate.
pub type ScreenPosition = Position<i32, Screen>;

#[inline]
fn to_logical_value<T>(a: T, dpi: T) -> T
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    a * num::cast(DEFAULT_DPI).unwrap() / dpi
}

#[inline]
fn to_physical_value<T>(a: T, dpi: T) -> T
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    a * dpi / num::cast(DEFAULT_DPI).unwrap()
}

impl<T> Position<T, Logical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    pub fn to_physical(&self, dpi: T) -> Position<T, Physical> {
        Position::new(
            to_physical_value(self.x, dpi),
            to_physical_value(self.y, dpi),
        )
    }
}

impl<T> Position<T, Physical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    pub fn to_logical(&self, dpi: T) -> Position<T, Logical> {
        Position::new(to_logical_value(self.x, dpi), to_logical_value(self.y, dpi))
    }
}

impl<T> Size<T, Logical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    pub fn to_physical(&self, dpi: T) -> Size<T, Physical> {
        Size::new(
            to_physical_value(self.width, dpi),
            to_physical_value(self.height, dpi),
        )
    }
}

impl<T> Size<T, Physical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    pub fn to_logical(&self, dpi: T) -> Size<T, Logical> {
        Size::new(
            to_logical_value(self.width, dpi),
            to_logical_value(self.height, dpi),
        )
    }
}

/// Converts to a logical position.
pub trait ToLogicalPosition<T> {
    fn to_logical(&self, dpi: T) -> Position<T, Logical>;
}

impl<T> ToLogicalPosition<T> for Position<T, Logical>
where
    T: Copy,
{
    #[inline]
    fn to_logical(&self, _: T) -> Position<T, Logical> {
        *self
    }
}

impl<T> ToLogicalPosition<T> for Position<T, Physical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    fn to_logical(&self, dpi: T) -> Position<T, Logical> {
        self.to_logical(dpi)
    }
}

/// Converts to a logical size.
pub trait ToLogicalSize<T> {
    fn to_logical(&self, dpi: T) -> Size<T, Logical>;
}

impl<T> ToLogicalSize<T> for Size<T, Logical>
where
    T: Copy,
{
    #[inline]
    fn to_logical(&self, _: T) -> Size<T, Logical> {
        *self
    }
}

impl<T> ToLogicalSize<T> for Size<T, Physical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    fn to_logical(&self, dpi: T) -> Size<T, Logical> {
        self.to_logical(dpi)
    }
}

/// Converts to a physical position.
pub trait ToPhysicalPosition<T> {
    fn to_physical(&self, dpi: T) -> Position<T, Physical>;
}

impl<T> ToPhysicalPosition<T> for Position<T, Logical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    fn to_physical(&self, dpi: T) -> Position<T, Physical> {
        self.to_physical(dpi)
    }
}

impl<T> ToPhysicalPosition<T> for Position<T, Physical>
where
    T: Copy,
{
    #[inline]
    fn to_physical(&self, _: T) -> Position<T, Physical> {
        *self
    }
}

/// Converts to a physical size.
pub trait ToPhysicalSize<T> {
    fn to_physical(&self, dpi: T) -> Size<T, Physical>;
}

impl<T> ToPhysicalSize<T> for Size<T, Logical>
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + Copy + num::NumCast,
{
    #[inline]
    fn to_physical(&self, dpi: T) -> Size<T, Physical> {
        self.to_physical(dpi)
    }
}

impl<T> ToPhysicalSize<T> for Size<T, Physical>
where
    T: Copy,
{
    #[inline]
    fn to_physical(&self, _: T) -> Size<T, Physical> {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_position() {
        let src = LogicalPosition::new(128.0, 256.0);
        let dst = src.cast::<i32>();
        assert!(dst.x == 128);
        assert!(dst.y == 256);
    }

    #[test]
    fn cast_size() {
        let src = LogicalSize::new(128.0, 256.0);
        let dst = src.cast::<i32>();
        assert!(dst.width == 128);
        assert!(dst.height == 256);
    }

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
    
    #[test]
    #[cfg(feature = "serde")]
    fn serde_check() {
        let src = PhysicalPosition::new(128, 256);
        let dest: PhysicalPosition<i32> = serde_json::from_str(&serde_json::to_string(&src).unwrap()).unwrap();
        assert!(dest.x == 128);
        assert!(dest.y == 256);
        let src = PhysicalSize::new(128, 256);
        let dest: PhysicalSize<i32> = serde_json::from_str(&serde_json::to_string(&src).unwrap()).unwrap();
        assert!(dest.width == 128);
        assert!(dest.height == 256);
        let src = LogicalPosition::new(128, 256);
        let dest: LogicalPosition<i32> = serde_json::from_str(&serde_json::to_string(&src).unwrap()).unwrap();
        assert!(dest.x == 128);
        assert!(dest.y == 256);
        let src = LogicalSize::new(128, 256);
        let dest: LogicalSize<i32> = serde_json::from_str(&serde_json::to_string(&src).unwrap()).unwrap();
        assert!(dest.width == 128);
        assert!(dest.height == 256);
        let src = ScreenPosition::new(128, 256);
        let dest: ScreenPosition = serde_json::from_str(&serde_json::to_string(&src).unwrap()).unwrap();
        assert!(dest.x == 128);
        assert!(dest.y == 256)
    }
}
