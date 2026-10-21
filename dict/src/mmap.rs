// SPDX-License-Identifier: GPL-3.0-or-later
//! **OS-native mmap** tu dien - FFI thuan, khong dependency.
//!
//! - Windows: `CreateFileW` + `CreateFileMappingW` + `MapViewOfFile`
//!   (`FILE_MAP_READ`) + `UnmapViewOfFile`.
//! - macOS / Linux (va BSD): `open` + `mmap(PROT_READ, MAP_PRIVATE)` +
//!   `munmap`.
//!
//! Buffer tra ve la **read-only** va troi (dang process) - dung de tra ve
//! `Arc` cho moi client.

use std::io;

/// Loi khi map file tu dien.
#[derive(Debug)]
pub enum MapError {
    /// Khong mo duoc file.
    Io(io::Error),
    /// File khong phai FST hop le / header sai.
    InvalidFormat(&'static str),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::Io(e) => write!(f, "io error: {e}"),
            MapError::InvalidFormat(msg) => write!(f, "invalid dictionary format: {msg}"),
        }
    }
}

impl std::error::Error for MapError {}

impl From<io::Error> for MapError {
    fn from(e: io::Error) -> Self {
        MapError::Io(e)
    }
}

/// Buffer da map (read-only).
#[derive(Debug)]
pub struct Mapping {
    ptr: *const u8,
    len: usize,
}

// Safety: buffer chi doc, khong bao gio bi ghi; `Drop` unmap dung 1 lan.
unsafe impl Send for Mapping {}
unsafe impl Sync for Mapping {}

impl Mapping {
    /// Do dai buffer (byte).
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` neu buffer rong.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Xem buffer.
    pub fn as_slice(&self) -> &[u8] {
        // Safety: `ptr`/`len` hop le (mapping con song).
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Map file FST tu duong dan.
    pub fn open(path: &std::path::Path) -> Result<Self, MapError> {
        platform::map_file(path)
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        platform::unmap(self.ptr, self.len);
    }
}

/// Do dai file (byte) - dung de biet mang truoc khi map.
fn file_len(path: &std::path::Path) -> Result<u64, MapError> {
    Ok(std::fs::metadata(path)?.len())
}

#[cfg(windows)]
mod platform {
    use super::MapError;
    use std::ffi::c_void;
    use std::path::Path;
    use std::ptr;

    type Handle = isize;

    const INVALID_HANDLE_VALUE: Handle = -1;
    const GENERIC_READ: u32 = 0x8000_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const OPEN_EXISTING: u32 = 3;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
    const PAGE_READONLY: u32 = 0x02;
    /// `FILE_MAP_READ` = 0x0004 (khong phai 0x0004_0000 - do la bit cua
    /// `SECURITY_ATTRIBUTES`; sai gia tri -> OS map view voi `PAGE_NOACCESS`
    /// -> doc bi `STATUS_ACCESS_VIOLATION`).
    const FILE_MAP_READ: u32 = 0x0004;

    extern "system" {
        fn CreateFileW(
            lpFileName: *const u16,
            dwDesiredAccess: u32,
            dwShareMode: u32,
            lpSecurityAttributes: *mut c_void,
            dwCreationDisposition: u32,
            dwFlagsAndAttributes: u32,
            hTemplateFile: Handle,
        ) -> Handle;
        fn CreateFileMappingW(
            hFile: Handle,
            lpFileMappingAttributes: *mut c_void,
            flProtect: u32,
            dwMaximumSizeHigh: u32,
            dwMaximumSizeLow: u32,
            lpName: *const u16,
        ) -> Handle;
        fn MapViewOfFile(
            hFileMappingObject: Handle,
            dwDesiredAccess: u32,
            dwFileOffsetHigh: u32,
            dwFileOffsetLow: u32,
            dwNumberOfBytesToMap: usize,
        ) -> *mut c_void;
        fn UnmapViewOfFile(lpBaseAddress: *mut c_void) -> i32;
        fn CloseHandle(hObject: Handle) -> i32;
    }

    pub fn map_file(path: &Path) -> Result<super::Mapping, MapError> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let file = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                0,
            )
        };
        if file == INVALID_HANDLE_VALUE {
            return Err(MapError::Io(std::io::Error::last_os_error()));
        }

        // Kiem tra kich thuoc **truoc** khi tao mapping: file rong khong tao
        // duoc view -> bao `InvalidFormat` (khong phai loi I/O mo file).
        let len = super::file_len(path)? as usize;
        if len == 0 {
            unsafe { CloseHandle(file) };
            return Err(MapError::InvalidFormat("file rong"));
        }

        let mapping =
            unsafe { CreateFileMappingW(file, ptr::null_mut(), PAGE_READONLY, 0, 0, ptr::null()) };
        unsafe { CloseHandle(file) };
        if mapping == 0 {
            return Err(MapError::Io(std::io::Error::last_os_error()));
        }

        // `dwNumberOfBytesToMap = len` -> map dung kich thuoc file.
        let view = unsafe { MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, len) };
        unsafe { CloseHandle(mapping) };
        if view.is_null() {
            return Err(MapError::Io(std::io::Error::last_os_error()));
        }

        Ok(super::Mapping {
            ptr: view.cast::<u8>(),
            len,
        })
    }

    pub fn unmap(ptr: *const u8, _len: usize) {
        if !ptr.is_null() {
            unsafe {
                UnmapViewOfFile(ptr as *mut c_void);
            }
        }
    }
}

#[cfg(unix)]
mod platform {
    use super::MapError;
    use std::os::raw::{c_int, c_void};
    use std::path::Path;
    use std::ptr;

    const PROT_READ: c_int = 1;
    const MAP_PRIVATE: c_int = 2;

    extern "C" {
        // #[link_name] giu ten symbol libc goc (clippy moi cam ten open/close).
        #[link_name = "open"]
        fn open_file(path: *const u8, flags: c_int, ...) -> c_int;
        #[link_name = "close"]
        fn close_fd(fd: c_int) -> c_int;
        fn mmap(
            addr: *mut c_void,
            length: usize,
            prot: c_int,
            flags: c_int,
            fd: c_int,
            offset: i64,
        ) -> *mut c_void;
        fn munmap(addr: *mut c_void, length: usize) -> c_int;
    }

    pub fn map_file(path: &Path) -> Result<super::Mapping, MapError> {
        use std::os::unix::ffi::OsStrExt;
        let mut bytes = path.as_os_str().as_bytes().to_vec();
        bytes.push(0);
        let fd = unsafe { open_file(bytes.as_ptr(), 0) };
        if fd < 0 {
            return Err(MapError::Io(std::io::Error::last_os_error()));
        }
        let len = super::file_len(path)? as usize;
        if len == 0 {
            unsafe { close_fd(fd) };
            return Err(MapError::InvalidFormat("file rong"));
        }
        let view = unsafe { mmap(ptr::null_mut(), len, PROT_READ, MAP_PRIVATE, fd, 0) };
        unsafe { close_fd(fd) };
        if view.is_null() || view as isize == -1 {
            return Err(MapError::Io(std::io::Error::last_os_error()));
        }
        Ok(super::Mapping {
            ptr: view.cast::<u8>(),
            len,
        })
    }

    pub fn unmap(ptr: *const u8, len: usize) {
        if !ptr.is_null() {
            unsafe {
                munmap(ptr as *mut c_void, len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ghi file tam va doc lai qua mapping - kiem tra view doc duoc.
    #[test]
    fn mapping_doc_byte_dau() {
        let dir = std::env::temp_dir().join("textvn-mmap-test");
        std::fs::create_dir_all(&dir).expect("tao thu muc temp");
        let path = dir.join("t.bin");
        std::fs::write(&path, b"hello world").expect("ghi file");

        let m = Mapping::open(&path).expect("map file");
        assert_eq!(m.len(), 11);
        assert_eq!(m.as_slice(), b"hello world");

        let _ = std::fs::remove_file(&path);
    }

    /// File khong ton tai -> loi I/O (khong panic).
    #[test]
    fn mapping_file_khong_ton_tai() {
        let path = std::env::temp_dir().join("textvn-mmap-test/khong-co-that.bin");
        assert!(Mapping::open(&path).is_err());
    }

    /// File rong -> `InvalidFormat`.
    #[test]
    fn mapping_file_rong() {
        let dir = std::env::temp_dir().join("textvn-mmap-test");
        std::fs::create_dir_all(&dir).expect("tao thu muc temp");
        let path = dir.join("rong.bin");
        std::fs::write(&path, b"").expect("ghi file");

        let err = Mapping::open(&path).expect_err("file rong phai bi tu choi");
        assert!(matches!(err, MapError::InvalidFormat(_)));

        let _ = std::fs::remove_file(&path);
    }
}
