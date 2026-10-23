//! NV12 video frame representation, scaling, framing and color conversion.
//! High-performance, zero-copy low-latency video pipeline for virtual camera blitting.

pub const BLACK_Y: u8 = 16;
pub const NEUTRAL_UV: u8 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// Crops to fill the output frame.
    Fill,
    /// Shows the full picture with letterbox / pillarbox bars.
    Fit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nv12 {
    pub width: usize,
    pub height: usize,
    /// Y plane followed by interleaved U and V at half resolution (width * height * 3 / 2 bytes).
    pub data: Vec<u8>,
}

impl Nv12 {
    pub fn black(width: usize, height: usize) -> Self {
        let mut data = vec![BLACK_Y; width * height];
        data.resize(width * height * 3 / 2, NEUTRAL_UV);
        Self {
            width,
            height,
            data,
        }
    }

    pub fn len_for(width: usize, height: usize) -> usize {
        width * height * 3 / 2
    }

    pub fn y(&self) -> &[u8] {
        &self.data[..self.width * self.height]
    }

    pub fn uv(&self) -> &[u8] {
        &self.data[self.width * self.height..]
    }

    /// Scales this picture into `dst` (whose size is preserved), cropped or letterboxed, optionally mirrored.
    pub fn shape_into(&self, dst: &mut Nv12, framing: Framing, mirror: bool) {
        let (dw, dh) = (dst.width, dst.height);
        dst.data.resize(Nv12::len_for(dw, dh), 0);
        let (sx, sy, sw, sh, ox, oy, ow, oh) = place(self.width, self.height, dw, dh, framing);
        let (y_dst, uv_dst) = dst.data.split_at_mut(dw * dh);
        y_dst.fill(BLACK_Y);
        uv_dst.fill(NEUTRAL_UV);

        let src_y = self.y();
        for oy_i in 0..oh {
            let fy = (oy_i as f32 + 0.5) * sh as f32 / oh as f32 - 0.5;
            let y0 = (fy.max(0.0) as usize).min(sh - 1);
            let y1 = (y0 + 1).min(sh - 1);
            let wy = ((fy - y0 as f32).clamp(0.0, 1.0) * 256.0) as u32;
            let row0 = &src_y[(sy + y0) * self.width..];
            let row1 = &src_y[(sy + y1) * self.width..];
            let out_row = &mut y_dst[(oy + oy_i) * dw..];
            for ox_i in 0..ow {
                let fx = (ox_i as f32 + 0.5) * sw as f32 / ow as f32 - 0.5;
                let x0 = (fx.max(0.0) as usize).min(sw - 1);
                let x1 = (x0 + 1).min(sw - 1);
                let wx = ((fx - x0 as f32).clamp(0.0, 1.0) * 256.0) as u32;
                let (a, b) = (row0[sx + x0] as u32, row0[sx + x1] as u32);
                let (c, d) = (row1[sx + x0] as u32, row1[sx + x1] as u32);
                let top = a * (256 - wx) + b * wx;
                let bottom = c * (256 - wx) + d * wx;
                let v = (top * (256 - wy) + bottom * wy) >> 16;
                let dx = if mirror {
                    dw - 1 - (ox + ox_i)
                } else {
                    ox + ox_i
                };
                out_row[dx] = v as u8;
            }
        }

        let src_uv = self.uv();
        let (ow2, oh2, ox2, oy2) = (ow / 2, oh / 2, ox / 2, oy / 2);
        for j in 0..oh2 {
            let sj = (sy / 2 + j * (sh / 2) / oh2.max(1)).min(self.height / 2 - 1);
            for i in 0..ow2 {
                let si = (sx / 2 + i * (sw / 2) / ow2.max(1)).min(self.width / 2 - 1);
                let di = if mirror {
                    dw / 2 - 1 - (ox2 + i)
                } else {
                    ox2 + i
                };
                let s = sj * self.width + si * 2;
                let d = (oy2 + j) * dw + di * 2;
                uv_dst[d] = src_uv[s];
                uv_dst[d + 1] = src_uv[s + 1];
            }
        }
    }

    /// Converts NV12 into BGR24 (3 bytes per pixel, top row first).
    pub fn to_bgr(&self, out: &mut Vec<u8>) {
        out.resize(self.width * self.height * 3, 0);
        let (y, uv) = (self.y(), self.uv());
        for row in 0..self.height {
            for col in 0..self.width {
                let yy = (y[row * self.width + col] as i32 - 16).max(0) * 298;
                let k = (row / 2) * self.width + (col / 2) * 2;
                let (u, v) = (uv[k] as i32 - 128, uv[k + 1] as i32 - 128);
                let r = (yy + 409 * v + 128) >> 8;
                let g = (yy - 100 * u - 208 * v + 128) >> 8;
                let b = (yy + 516 * u + 128) >> 8;
                let o = (row * self.width + col) * 3;
                out[o] = b.clamp(0, 255) as u8;
                out[o + 1] = g.clamp(0, 255) as u8;
                out[o + 2] = r.clamp(0, 255) as u8;
            }
        }
    }

    /// Creates an NV12 frame from a BGRA pixel buffer.
    pub fn from_bgra(width: usize, height: usize, bgra: &[u8]) -> Self {
        let mut f = Nv12::black(width, height);
        let (y, uv) = f.data.split_at_mut(width * height);
        for row in 0..height {
            for col in 0..width {
                let p = &bgra[(row * width + col) * 4..];
                let (b, g, r) = (p[0] as i32, p[1] as i32, p[2] as i32);
                y[row * width + col] = (((66 * r + 129 * g + 25 * b + 128) >> 8) + 16) as u8;
                if row % 2 == 0 && col % 2 == 0 {
                    let k = (row / 2) * width + col;
                    uv[k] = (((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128) as u8;
                    uv[k + 1] = (((112 * r - 94 * g - 18 * b + 128) >> 8) + 128) as u8;
                }
            }
        }
        f
    }

    /// Renders a modern, calm standby placeholder for LinkCam.
    pub fn render_placeholder(width: usize, height: usize) -> Self {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::{COLORREF, RECT};
            use windows::Win32::Graphics::Gdi::{
                BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
                CreateCompatibleDC, CreateDIBSection, CreateFontW, DEFAULT_CHARSET, DIB_RGB_COLORS,
                DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, FF_SWISS,
                FW_SEMIBOLD, GdiFlush, HGDIOBJ, OUT_DEFAULT_PRECIS, SelectObject, SetBkMode,
                SetTextColor, TRANSPARENT,
            };
            use windows::core::w;

            unsafe {
                let dc = CreateCompatibleDC(None);
                let info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
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
                if let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                    let old_bitmap = SelectObject(dc, HGDIOBJ(bitmap.0));
                    let font = CreateFontW(
                        -(height as i32 / 24),
                        0,
                        0,
                        0,
                        FW_SEMIBOLD.0 as i32,
                        0,
                        0,
                        0,
                        DEFAULT_CHARSET.0 as u32,
                        OUT_DEFAULT_PRECIS.0 as u32,
                        CLIP_DEFAULT_PRECIS.0 as u32,
                        CLEARTYPE_QUALITY.0 as u32,
                        FF_SWISS.0 as u32,
                        w!("Segoe UI"),
                    );
                    let old_font = SelectObject(dc, HGDIOBJ(font.0));
                    // Crisp slate text on dark background
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
                    let frame = Nv12::from_bgra(width, height, pixels);
                    SelectObject(dc, old_font);
                    SelectObject(dc, old_bitmap);
                    let _ = DeleteObject(HGDIOBJ(font.0));
                    let _ = DeleteObject(HGDIOBJ(bitmap.0));
                    let _ = DeleteDC(dc);
                    return frame;
                }
                let _ = DeleteDC(dc);
            }
        }

        Nv12::black(width, height)
    }
}

/// Source window (x, y, w, h) and output window (x, y, w, h), aligned to even bounds.
fn place(
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
    framing: Framing,
) -> (usize, usize, usize, usize, usize, usize, usize, usize) {
    let even = |v: usize| (v / 2 * 2).max(2);
    let src_wider = sw * dh > dw * sh;
    match (framing, src_wider) {
        (Framing::Fill, true) => {
            let w = even(sh * dw / dh);
            (even((sw - w) / 2).min(sw - w), 0, w, sh, 0, 0, dw, dh)
        }
        (Framing::Fill, false) => {
            let h = even(sw * dh / dw);
            (0, even((sh - h) / 2).min(sh - h), sw, h, 0, 0, dw, dh)
        }
        (Framing::Fit, true) => {
            let h = even(dw * sh / sw);
            (0, 0, sw, sh, 0, even((dh - h) / 2).min(dh - h), dw, h)
        }
        (Framing::Fit, false) => {
            let w = even(dh * sw / sh);
            (0, 0, sw, sh, even((dw - w) / 2).min(dw - w), 0, w, dh)
        }
    }
}
