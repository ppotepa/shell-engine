use engine_core::color::Color;

pub fn to_sdl_color(color: Color) -> sdl2::pixels::Color {
    let (r, g, b) = color.to_rgb();
    sdl2::pixels::Color::RGB(r, g, b)
}
