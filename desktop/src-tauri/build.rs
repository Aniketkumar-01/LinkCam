use std::fs;
use std::path::Path;

fn ensure_icons() {
    let icons_dir = Path::new("icons");
    let _ = fs::create_dir_all(icons_dir);

    let ico_path = icons_dir.join("icon.ico");
    if !ico_path.exists() {
        let mut ico_data = Vec::new();
        // ICONDIR: reserved 0, type 1 (ico), count 1
        ico_data.extend_from_slice(&[0, 0, 1, 0, 1, 0]);

        let width: u8 = 16;
        let height: u8 = 16;
        let bpp: u16 = 32;
        let header_size: u32 = 40;
        let xor_size: u32 = (width as u32) * (height as u32) * 4;
        let and_size: u32 = ((width as u32 + 31) / 32 * 4) * (height as u32);
        let res_size = header_size + xor_size + and_size;
        let offset: u32 = 6 + 16;

        // ICONDIRENTRY (16 bytes)
        ico_data.push(width);
        ico_data.push(height);
        ico_data.push(0); // colors
        ico_data.push(0); // reserved
        ico_data.extend_from_slice(&1u16.to_le_bytes()); // planes
        ico_data.extend_from_slice(&bpp.to_le_bytes()); // bit count
        ico_data.extend_from_slice(&res_size.to_le_bytes()); // resource size
        ico_data.extend_from_slice(&offset.to_le_bytes()); // image offset

        // BITMAPINFOHEADER (40 bytes)
        ico_data.extend_from_slice(&header_size.to_le_bytes());
        ico_data.extend_from_slice(&(width as i32).to_le_bytes());
        ico_data.extend_from_slice(&((height as i32) * 2).to_le_bytes()); // biHeight is double
        ico_data.extend_from_slice(&1u16.to_le_bytes()); // planes
        ico_data.extend_from_slice(&bpp.to_le_bytes()); // bit count
        ico_data.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
        ico_data.extend_from_slice(&(xor_size + and_size).to_le_bytes()); // biSizeImage
        ico_data.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
        ico_data.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
        ico_data.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
        ico_data.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

        // XOR pixel data (BGRA): 16x16 pixels with cyan camera aperture circle
        for y in 0..height {
            for x in 0..width {
                let dx = (x as i32) - 8;
                let dy = (y as i32) - 8;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= 36 {
                    // Electric Cyan #00e5ff
                    ico_data.extend_from_slice(&[0xff, 0xe5, 0x00, 0xff]);
                } else if dist_sq <= 56 {
                    // Deep navy #0c192c
                    ico_data.extend_from_slice(&[0x2c, 0x19, 0x0c, 0xff]);
                } else {
                    // Transparent
                    ico_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                }
            }
        }

        // AND mask (opaque = 0, transparent = 1)
        let row_bytes = ((width as usize + 31) / 32) * 4;
        for y in 0..height {
            for _ in 0..row_bytes {
                let mut mask_byte = 0u8;
                for bit in 0..8 {
                    let x = bit;
                    let dx = (x as i32) - 8;
                    let dy = (y as i32) - 8;
                    if dx * dx + dy * dy > 56 {
                        mask_byte |= 1 << (7 - bit);
                    }
                }
                ico_data.push(mask_byte);
            }
        }

        let _ = fs::write(&ico_path, &ico_data);
    }
}

fn main() {
    ensure_icons();
    tauri_build::build();
}
