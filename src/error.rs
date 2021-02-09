#[macro_export]
macro_rules! last_error {
    ($s:literal) => {
        ::log::error!(
            "{}({}) {}: 0x{:<08x}",
            file!(),
            line!(),
            $s,
            ::winapi::um::errhandlingapi::GetLastError()
        )
    };
}
