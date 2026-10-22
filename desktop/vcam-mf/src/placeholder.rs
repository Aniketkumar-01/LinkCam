//! Standby placeholder image shown when no phone stream is active.
//! Drawn once with Windows GDI and converted into standard NV12 format.

use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, FF_SWISS, FW_SEMIBOLD, GdiFlush,
    OUT_DEFAULT_PRECIS, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::core::w;

pub struct PlaceholderFrame {
    pub data: Vec<u8>,
}

pub fn render(width: usize, height: usize) -> PlaceholderFrame {
    unsafe {
        let dc = CreateCompatibleDC(None);
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
            let _ = DeleteDC(dc);
            return PlaceholderFrame { data: vec![16; width * height * 3 / 2] };
        };
        let old_bitmap = SelectObject(dc, bitmap.into());
        let font = CreateFontW(
            -(height as i32 / 24),
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            FF_SWISS.0 as u32,
            w!("Segoe UI"),
        );
        let old_font = SelectObject(dc, font.into());
        // Clean silver-blue text on dark slate
        SetTextColor(dc, COLORREF(0x00E0D0C0));
        SetBkMode(dc, TRANSPARENT);
        let mut text: Vec<u16> = "LinkCam \u{00B7} Connect phone in LinkCam app"
            .encode_utf16()
            .collect();
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        DrawTextW(
            dc,
            &mut text,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        let _ = GdiFlush();
        let pixels = std::slice::from_raw_parts(bits as *const u8, width * height * 4);

        // Convert BGRA pixels to NV12
        let mut nv12 = vec![16u8; width * height];
        nv12.resize(width * height * 3 / 2, 128);
        let (y, uv) = nv12.split_at_mut(width * height);
        for row in 0..height {
            for col in 0..width {
                let p = &pixels[(row * width + col) * 4..];
                let (b, g, r) = (p[0] as i32, p[1] as i32, p[2] as i32);
                y[row * width + col] = (((66 * r + 129 * g + 25 * b + 128) >> 8) + 16) as u8;
                if row % 2 == 0 && col % 2 == 0 {
                    let k = (row / 2) * width + col;
                    uv[k] = (((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128) as u8;
                    uv[k + 1] = (((112 * r - 94 * g - 18 * b + 128) >> 8) + 128) as u8;
                }
            }
        }

        SelectObject(dc, old_font);
        SelectObject(dc, old_bitmap);
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        PlaceholderFrame { data: nv12 }
    }
}
