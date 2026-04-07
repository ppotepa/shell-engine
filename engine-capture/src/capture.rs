//! Frame capture for visual regression testing — serializes buffer state to binary files.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use engine_core::buffer::{Buffer, Cell};
use engine_error::EngineError;

/// Frame capture writer — serializes buffer state to binary files for regression testing.
pub struct FrameCapture {
    output_dir: PathBuf,
    frame_num: u32,
}

impl FrameCapture {
    /// Create a new frame capture writer, initializing output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Result<Self, EngineError> {
        let output_dir = output_dir.into();
        fs::create_dir_all(&output_dir).map_err(EngineError::Render)?;
        Ok(Self {
            output_dir,
            frame_num: 0,
        })
    }

    /// Capture the current buffer state to a binary file.
    ///
    /// Format: [width:u16][height:u16][cells...]
    /// Each cell: [symbol:u32 LE][fg_r:u8][fg_g:u8][fg_b:u8][bg_r:u8][bg_g:u8][bg_b:u8] (10 bytes)
    pub fn capture(&mut self, buffer: &Buffer) -> Result<(), EngineError> {
        let filename = format!("frame_{:06}.bin", self.frame_num);
        let path = self.output_dir.join(&filename);

        let mut file = fs::File::create(&path).map_err(EngineError::Render)?;

        // Write header
        file.write_all(&buffer.width.to_le_bytes())
            .map_err(EngineError::Render)?;
        file.write_all(&buffer.height.to_le_bytes())
            .map_err(EngineError::Render)?;

        // Write cells
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                if let Some(cell) = buffer.get(x, y) {
                    write_cell(&mut file, cell)?;
                } else {
                    write_cell(&mut file, &Cell::default())?;
                }
            }
        }

        self.frame_num = self.frame_num.wrapping_add(1);
        Ok(())
    }
}

fn write_cell(file: &mut fs::File, cell: &Cell) -> Result<(), EngineError> {
    // Write symbol as u32 LE
    let symbol_code = cell.symbol as u32;
    file.write_all(&symbol_code.to_le_bytes())
        .map_err(EngineError::Render)?;

    // Write fg color as RGB
    let (fg_r, fg_g, fg_b) = color_to_rgb(cell.fg);
    file.write_all(&[fg_r, fg_g, fg_b])
        .map_err(EngineError::Render)?;

    // Write bg color as RGB
    let (bg_r, bg_g, bg_b) = color_to_rgb(cell.bg);
    file.write_all(&[bg_r, bg_g, bg_b])
        .map_err(EngineError::Render)?;

    Ok(())
}

fn color_to_rgb(color: engine_core::color::Color) -> (u8, u8, u8) {
    use engine_core::color::Color;
    match color {
        Color::Reset => (0, 0, 0),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Yellow => (255, 255, 0),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::Grey => (128, 128, 128),
        Color::DarkGrey => (64, 64, 64),
        Color::DarkRed => (128, 0, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::DarkBlue => (0, 0, 128),
        Color::DarkYellow => (128, 128, 0),
        Color::DarkMagenta => (128, 0, 128),
        Color::DarkCyan => (0, 128, 128),
        Color::Rgb { r, g, b } => (r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_capture() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let mut capture = FrameCapture::new(temp_dir.path()).expect("create capture");

        let buffer = Buffer::new(10, 10);
        capture.capture(&buffer).expect("capture frame");

        let frame_file = temp_dir.path().join("frame_000000.bin");
        assert!(frame_file.exists(), "frame file should exist");
    }

    #[test]
    fn test_color_to_rgb() {
        use engine_core::color::Color;
        assert_eq!(color_to_rgb(Color::Black), (0, 0, 0));
        assert_eq!(color_to_rgb(Color::White), (255, 255, 255));
        assert_eq!(
            color_to_rgb(Color::Rgb {
                r: 100,
                g: 150,
                b: 200
            }),
            (100, 150, 200)
        );
    }
}
