#![allow(clippy::missing_safety_doc)]

use std::{error::Error, io::Cursor, mem::forget, ptr::null_mut};

use jxl_oxide::{JxlImage, PixelFormat};

pub struct JxlOxidePy {
    pub keyframe: jxl_oxide::Render,
    pub width: u32,
    pub height: u32,
    pub pixfmt: jxl_oxide::PixelFormat,
}

#[repr(C)]
pub struct Array {
    data: *mut u8,
    len: usize,
}

#[no_mangle]
pub unsafe extern "C" fn new(val: *const u8, n: usize) -> *mut JxlOxidePy {
    let slice = unsafe { std::slice::from_raw_parts(val, n) };
    let Ok(decoded) = read_jxl(slice) else {
        return null_mut();
    };

    Box::into_raw(Box::new(decoded))
}

#[no_mangle]
pub unsafe extern "C" fn width(ptr: *mut JxlOxidePy) -> u32 {
    (*ptr).width
}

#[no_mangle]
pub unsafe extern "C" fn height(ptr: *mut JxlOxidePy) -> u32 {
    (*ptr).height
}

#[no_mangle]
pub unsafe extern "C" fn colorspace(ptr: *mut JxlOxidePy) -> *const u8 {
    if ptr.is_null() {
        return null_mut();
    }
    let d = &*ptr;
    let res = match d.pixfmt {
        PixelFormat::Gray => "L\0",
        PixelFormat::Graya => "LA\0",
        PixelFormat::Rgb => "RGB\0",
        PixelFormat::Rgba => "RGBA\0",
        PixelFormat::Cmyk => todo!(),
        PixelFormat::Cmyka => todo!(),
    };
    res.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn image(ptr: *mut JxlOxidePy) -> *mut Array {
    if ptr.is_null() {
        return null_mut();
    }
    let d = &*ptr;
    let fb = d.keyframe.image();
    let mut buf = vec![0u8; fb.width() * fb.height() * fb.channels()];
    for (b, s) in buf.iter_mut().zip(fb.buf()) {
        *b = (*s * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    }
    let res = Array {
        data: buf.as_mut_ptr(),
        len: buf.len(),
    };
    forget(buf);
    Box::into_raw(Box::new(res))
}

#[no_mangle]
pub unsafe extern "C" fn free_jxl_oxide(ptr: *mut JxlOxidePy) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _: Box<JxlOxidePy> = Box::from_raw(ptr);
        // drop(Vec::from_raw_parts(d.image, d.image_len, d.image_len));
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_array(ptr: *mut Array) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let array: Box<Array> = Box::from_raw(ptr);
        drop(Vec::from_raw_parts(array.data, array.len, array.len));
    }
}

fn read_jxl(bytes: &[u8]) -> Result<JxlOxidePy, Box<dyn Error + Send + Sync>> {
    let cursor = Cursor::new(bytes);
    println!("Start decoding");
    let mut image = JxlImage::from_reader(cursor)?;
    let size = &image.image_header().size;
    let width = size.width;
    let height = size.height;

    let mut renderer = image.renderer();
    let pixfmt = renderer.pixel_format();

    let result = renderer.render_next_frame()?;
    let keyframe = match result {
        jxl_oxide::RenderResult::Done(frame) => frame,
        _ => return Err("Unexpected end of JXL file".into()),
    };

    let decoded = JxlOxidePy {
        keyframe,
        width,
        height,
        pixfmt,
    };

    println!("Return");
    Ok(decoded)
}
