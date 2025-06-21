use anyhow::{anyhow, Context as _};
use image::GrayImage;
use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;

#[allow(non_camel_case_types)]
type zbar_image_t = c_void;
#[allow(non_camel_case_types)]
type zbar_image_scanner_t = c_void;

#[repr(C)]
struct ZBarAPI<'lib> {
    zbar_image_create: Symbol<'lib, unsafe extern "C" fn() -> *mut zbar_image_t>,
    zbar_image_set_format: Symbol<'lib, unsafe extern "C" fn(*mut zbar_image_t, c_uint)>,
    zbar_image_set_size: Symbol<'lib, unsafe extern "C" fn(*mut zbar_image_t, c_uint, c_uint)>,
    zbar_image_set_data: Symbol<
        'lib,
        unsafe extern "C" fn(
            *mut zbar_image_t,
            *const c_void,
            usize,
            Option<unsafe extern "C" fn(*mut c_void)>,
        ),
    >,
    zbar_image_destroy: Symbol<'lib, unsafe extern "C" fn(*mut zbar_image_t)>,

    zbar_image_scanner_create: Symbol<'lib, unsafe extern "C" fn() -> *mut zbar_image_scanner_t>,
    zbar_image_scanner_destroy: Symbol<'lib, unsafe extern "C" fn(*mut zbar_image_scanner_t)>,
    zbar_scan_image:
        Symbol<'lib, unsafe extern "C" fn(*mut zbar_image_scanner_t, *mut zbar_image_t) -> c_int>,
    zbar_image_first_symbol: Symbol<'lib, unsafe extern "C" fn(*const zbar_image_t) -> *mut c_void>,
    zbar_symbol_get_data: Symbol<'lib, unsafe extern "C" fn(*const c_void) -> *const c_char>,
}

impl<'lib> ZBarAPI<'lib> {
    unsafe fn load(lib: &'lib Library) -> Result<Self, ()> {
        macro_rules! load {
            ($s:ident) => {
                lib.get::<unsafe extern "C" fn()>(_symbol!(stringify!($s))).map_err(|_| ())?
            };
            ($s:ident, fn($($arg:ty),*) -> $ret:ty) => {
                lib.get::<unsafe extern "C" fn($($arg),*) -> $ret>(stringify!($s).as_bytes()).map_err(|_| ())?
            };
        }

        Ok(Self {
            zbar_image_create: lib.get(b"zbar_image_create\0").map_err(|_| ())?,
            zbar_image_set_format: lib.get(b"zbar_image_set_format\0").map_err(|_| ())?,
            zbar_image_set_size: lib.get(b"zbar_image_set_size\0").map_err(|_| ())?,
            zbar_image_set_data: lib.get(b"zbar_image_set_data\0").map_err(|_| ())?,
            zbar_image_destroy: lib.get(b"zbar_image_destroy\0").map_err(|_| ())?,
            zbar_image_scanner_create: lib.get(b"zbar_image_scanner_create\0").map_err(|_| ())?,
            zbar_image_scanner_destroy: lib.get(b"zbar_image_scanner_destroy\0").map_err(|_| ())?,
            zbar_scan_image: lib.get(b"zbar_scan_image\0").map_err(|_| ())?,
            zbar_image_first_symbol: lib.get(b"zbar_image_first_symbol\0").map_err(|_| ())?,
            zbar_symbol_get_data: lib.get(b"zbar_symbol_get_data\0").map_err(|_| ())?,
        })
    }
}

pub fn scan_qr_from_image(img: &GrayImage) -> anyhow::Result<String> {
    let lib = unsafe {
        Library::new("libzbar.so")
            .or_else(|_| Library::new("libzbar.dylib"))
            .map_err(anyhow::Error::new)
            .context("could not load ZBar library (tried .so and .dylib)")?
    };
    tracing::info!("Using ZBar library: {:?}", lib);

    let api = unsafe { ZBarAPI::load(&lib)
        .map_err(|_| anyhow!("Failed to load lib"))? };

    let (width, height) = img.dimensions();
    tracing::debug!("Width: {width:?}, Height {height:?}");
    let raw = img.clone().into_raw();

    unsafe {
        let zimg = (api.zbar_image_create)();
        (api.zbar_image_set_format)(zimg, u32::from_le_bytes(*b"Y800"));
        (api.zbar_image_set_size)(zimg, width, height);
        (api.zbar_image_set_data)(zimg, raw.as_ptr() as *const _, raw.len(), None);

        let scanner = (api.zbar_image_scanner_create)();
        (api.zbar_scan_image)(scanner, zimg);

        let symbol = (api.zbar_image_first_symbol)(zimg);
        if symbol.is_null() {
            (api.zbar_image_destroy)(zimg);
            (api.zbar_image_scanner_destroy)(scanner);
            return Err(anyhow::anyhow!("No QR code found in the image"));
        }

        let data = (api.zbar_symbol_get_data)(symbol);
        let result = CStr::from_ptr(data).to_string_lossy().into_owned();

        (api.zbar_image_destroy)(zimg);
        (api.zbar_image_scanner_destroy)(scanner);

        Ok(result)
    }
}
