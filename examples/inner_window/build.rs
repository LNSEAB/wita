fn main() {
    windows::build!(
        windows::win32::windows_and_messaging::*,
        windows::win32::direct2d::*,
        windows::win32::dxgi::*,
    );
}