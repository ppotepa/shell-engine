//! Frame comparison tool for regression testing — compares captured frames between two runs.

use std::fs;
use std::io::Read;
use std::path::Path;

/// Frame metadata (width, height)
#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub width: u16,
    pub height: u16,
}

/// Serialized cell data (symbol, fg_r, fg_g, fg_b, bg_r, bg_g, bg_b)
#[derive(Debug, Clone, PartialEq)]
pub struct SerializedCell {
    pub symbol: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
}

/// Load a single frame file and return header + cells
pub fn load_frame(path: &Path) -> std::io::Result<(FrameHeader, Vec<SerializedCell>)> {
    let mut file = fs::File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    if buf.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame file too small for header",
        ));
    }

    let width = u16::from_le_bytes([buf[0], buf[1]]);
    let height = u16::from_le_bytes([buf[2], buf[3]]);
    let expected_size = 4 + (width as usize) * (height as usize) * 10;

    if buf.len() < expected_size {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "frame truncated: expected {expected_size} bytes, got {}",
                buf.len()
            ),
        ));
    }

    let mut cells = Vec::new();
    let mut offset = 4;

    for _ in 0..(width as usize * height as usize) {
        let symbol = u32::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ]);
        let fg_r = buf[offset + 4];
        let fg_g = buf[offset + 5];
        let fg_b = buf[offset + 6];
        let bg_r = buf[offset + 7];
        let bg_g = buf[offset + 8];
        let bg_b = buf[offset + 9];

        cells.push(SerializedCell {
            symbol,
            fg_r,
            fg_g,
            fg_b,
            bg_r,
            bg_g,
            bg_b,
        });

        offset += 10;
    }

    Ok((FrameHeader { width, height }, cells))
}

/// Compare two frame files and return the first differing cell index, or None if identical
pub fn compare_frames(
    path1: &Path,
    path2: &Path,
) -> std::io::Result<Option<(usize, SerializedCell, SerializedCell)>> {
    let (header1, cells1) = load_frame(path1)?;
    let (header2, cells2) = load_frame(path2)?;

    // Check headers match
    if header1.width != header2.width || header1.height != header2.height {
        return Ok(Some((0, cells1[0].clone(), cells2[0].clone())));
    }

    // Compare cells
    for (i, (cell1, cell2)) in cells1.iter().zip(cells2.iter()).enumerate() {
        if cell1 != cell2 {
            return Ok(Some((i, cell1.clone(), cell2.clone())));
        }
    }

    Ok(None)
}

/// List all frame files in a directory (sorted by frame number)
pub fn list_frame_files(dir: &Path) -> std::io::Result<Vec<std::fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy().starts_with("frame_")
                && e.file_name().to_string_lossy().ends_with(".bin")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name().to_string_lossy().to_string());

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_roundtrip() {
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");

        // Write a simple 2x2 frame
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&2u16.to_le_bytes()).unwrap(); // width
        file.write_all(&2u16.to_le_bytes()).unwrap(); // height

        // Cell 0: 'A', fg=(255,0,0), bg=(0,255,0)
        let cell_data = [
            65u32.to_le_bytes().to_vec(),
            vec![255u8, 0, 0, 0, 255, 0], // fg_r,fg_g,fg_b,bg_r,bg_g,bg_b
        ]
        .concat();
        file.write_all(&cell_data).unwrap();

        // Cell 1-3: spaces (all zeros)
        for _ in 0..3 {
            file.write_all(&[0u8; 10]).unwrap();
        }

        let (header, cells) = load_frame(&path).unwrap();
        assert_eq!(header.width, 2);
        assert_eq!(header.height, 2);
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[0].symbol, 65);
        assert_eq!(cells[0].fg_r, 255);
        assert_eq!(cells[0].bg_g, 255);
    }
}
