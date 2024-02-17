#![allow(clippy::missing_safety_doc)]

mod errors;
pub use errors::*;

// # TODO
// * Safety docs
// * Handle bitdepths >8

use std::{error::Error, io::Cursor, mem::forget, ptr};

use jxl_oxide::{JxlImage, JxlThreadPool, PixelFormat};

pub struct JxlOxide {
    pub image: JxlImage,
    pub width: u32,
    pub height: u32,
    pub pixfmt: jxl_oxide::PixelFormat,
}

#[repr(C)]
pub struct Array {
    data: ptr::NonNull<u8>,
    len: usize,
}

#[no_mangle]
pub unsafe extern "C" fn new(val: *const u8, n: usize) -> *mut JxlOxide {
    let slice = std::slice::from_raw_parts(val, n);
    let decoded = match read_jxl(slice) {
        Ok(v) => v,
        Err(e) => {
            update_last_error(e);
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(decoded))
}

#[no_mangle]
pub unsafe extern "C" fn width(ptr: *mut JxlOxide) -> u32 {
    (*ptr).width
}

#[no_mangle]
pub unsafe extern "C" fn height(ptr: *mut JxlOxide) -> u32 {
    (*ptr).height
}

#[no_mangle]
pub unsafe extern "C" fn colorspace(ptr: *mut JxlOxide) -> *const u8 {
    let res = match (*ptr).pixfmt {
        PixelFormat::Gray => "L\0",
        PixelFormat::Graya => "LA\0",
        PixelFormat::Rgb => "RGB\0",
        PixelFormat::Rgba => "RGBA\0",
        PixelFormat::Cmyk => "CMYK\0",
        PixelFormat::Cmyka => return ptr::null(),
    };
    res.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn image(ptr: *mut JxlOxide) -> *mut Array {
    let fb = match (*ptr).image.render_frame(0) {
        Ok(render) => render.image(),
        Err(e) => {
            update_last_error(e);
            return ptr::null_mut();
        }
    };

    let mut buf = vec![0u8; fb.width() * fb.height() * fb.channels()];
    for (b, s) in buf.iter_mut().zip(fb.buf()) {
        *b = (*s * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    }

    let res = Array {
        data: ptr::NonNull::new(buf.as_mut_ptr()).unwrap(),
        len: buf.len(),
    };
    forget(buf);
    Box::into_raw(Box::new(res))
}

#[no_mangle]
pub unsafe extern "C" fn free_jxl_oxide(ptr: *mut JxlOxide) {
    if ptr.is_null() {
        return;
    }
    let _: Box<JxlOxide> = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn free_array(ptr: *mut Array) {
    if ptr.is_null() {
        return;
    }
    let array: Box<Array> = Box::from_raw(ptr);
    drop(Vec::from_raw_parts(
        array.data.as_ptr(),
        array.len,
        array.len,
    ));
}

fn read_jxl(bytes: &[u8]) -> Result<JxlOxide, Box<dyn Error + Send + Sync + 'static>> {
    let cursor = Cursor::new(bytes);
    let pool = JxlThreadPool::rayon(None);
    let image = JxlImage::builder().pool(pool).read(cursor)?;
    let size = &image.image_header().size;
    let width = size.width;
    let height = size.height;

    let pixfmt = image.pixel_format();

    let decoded = JxlOxide {
        image,
        width,
        height,
        pixfmt,
    };

    Ok(decoded)
}
