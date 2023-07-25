#![allow(clippy::missing_safety_doc)]

use std::{error::Error, io::Cursor, mem::forget, ptr::null_mut};

use jxl_oxide::JxlImage;

#[repr(C)]
pub struct JxlOxidePy {
    pub image: *mut u8,
    pub image_len: usize,
    pub width: u32,
    pub height: u32,
    pub pixfmt: PixelFormat,
}

#[repr(C)]
pub enum PixelFormat {
    Gray,
    Graya,
    Rgb,
    Rgba,
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
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe extern "C" fn pil_colorspace(ptr: *mut JxlOxidePy) -> *const u8 {
    if ptr.is_null() {
        return null_mut();
    }
    let d = &*ptr;
    let res = match d.pixfmt {
        PixelFormat::Gray => "L\0",
        PixelFormat::Graya => "LA\0",
        PixelFormat::Rgb => "RGB\0",
        PixelFormat::Rgba => "RGBA\0",
    };
    res.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut JxlOxidePy) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let d: Box<JxlOxidePy> = Box::from_raw(ptr);
        drop(Vec::from_raw_parts(d.image, d.image_len, d.image_len));
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

    let fb = keyframe.image();
    let mut buf = vec![0u8; fb.width() * fb.height() * fb.channels()];
    for (b, s) in buf.iter_mut().zip(fb.buf()) {
        *b = (*s * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    }

    let decoded = JxlOxidePy {
        image: buf.as_mut_ptr(),
        image_len: buf.len(),
        width,
        height,
        pixfmt: PixelFormat::from(pixfmt),
    };

    forget(buf);

    println!("Return");
    Ok(decoded)
}

impl From<jxl_oxide::PixelFormat> for PixelFormat {
    fn from(value: jxl_oxide::PixelFormat) -> Self {
        match value {
            jxl_oxide::PixelFormat::Gray => Self::Gray,
            jxl_oxide::PixelFormat::Graya => Self::Graya,
            jxl_oxide::PixelFormat::Rgb => Self::Rgb,
            jxl_oxide::PixelFormat::Rgba => Self::Rgba,
            jxl_oxide::PixelFormat::Cmyk => todo!(),
            jxl_oxide::PixelFormat::Cmyka => todo!(),
        }
    }
}
