//! Engine splash screen — shown once before the mod game loop starts.
//!
//! Loads the splash scene from `engine/assets/scenes/splash/scene.yml`, so the
//! image, audio, colours, and timing live in assets instead of code.

use std::collections::VecDeque;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::buffer::Buffer;
use crate::effects::{shared_dispatcher, Region};
use crate::scene::{HorizontalAlign, TermColour, VerticalAlign};
use base64::Engine as _;
use crossterm::{cursor, queue, style, terminal};
use engine_core::color::Color;
#[cfg(feature = "sdl2")]
use engine_render::OutputBackend;
use engine_render_terminal::color_convert;
use image::{imageops, load_from_memory};
use serde::Deserialize;
use serde_yaml::{Mapping, Value};

const SPLASH_SCENE_PATH: &str = "assets/scenes/splash/scene.yml";
#[cfg(feature = "sdl2")]
const SDL_SPLASH_SCALE_FACTOR: f32 = 0.5;

#[derive(Debug, Clone)]
pub struct SplashConfig {
    pub enabled: bool,
    pub scene_path: Option<PathBuf>,
}

impl Default for SplashConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scene_path: None,
        }
    }
}

/// Resolve splash startup configuration from `mod.yaml`.
///
/// Supported keys:
/// - `splash.enabled` (bool, default true)
/// - `splash.scene` (path to splash scene YAML, absolute from mod root or relative)
///   aliases: `scene-path`, `scene_path`
pub fn config_from_manifest(mod_source: &Path, manifest: &Value) -> SplashConfig {
    let Some(root) = manifest.as_mapping() else {
        return SplashConfig::default();
    };
    let Some(splash) = map_get(root, "splash").and_then(Value::as_mapping) else {
        return SplashConfig::default();
    };

    let enabled = map_get(splash, "enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let scene_path = map_get(splash, "scene")
        .or_else(|| map_get(splash, "scene-path"))
        .or_else(|| map_get(splash, "scene_path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(|raw| resolve_mod_path(mod_source, raw));

    SplashConfig {
        enabled,
        scene_path,
    }
}

fn map_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

fn resolve_mod_path(mod_source: &Path, raw: &str) -> PathBuf {
    let p = Path::new(raw);
    if p.is_absolute() {
        let trimmed = raw.trim_start_matches('/');
        mod_source.join(trimmed)
    } else {
        mod_source.join(p)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SplashSceneDefinition {
    id: String,
    title: String,
    image: String,
    splatter: Option<String>,
    audio: String,
    bg_colour: TermColour,
    alpha_threshold: u8,
    logo_scale: f32,
    #[serde(default)]
    logo_align_x: Option<HorizontalAlign>,
    #[serde(default)]
    logo_align_y: Option<VerticalAlign>,
    #[serde(default)]
    logo_offset_x_cells: i32,
    #[serde(default)]
    logo_offset_y_cells: i32,
    blank_ms: u64,
    logo_fade_in_ms: u64,
    shake_delay_ms: u64,
    shake_ms: u64,
    shake_amplitude_cells: i16,
    shake_rotate_deg: f32,
    shake_punch_scale: f32,
    splatter_delay_ms: u64,
    splatter_reveal_ms: u64,
    splatter_drip_ms: u64,
    splatter_scale: f32,
    #[serde(default)]
    splatter_align_x: Option<HorizontalAlign>,
    #[serde(default)]
    splatter_align_y: Option<VerticalAlign>,
    #[serde(default)]
    splatter_offset_x_cells: i32,
    #[serde(default)]
    splatter_offset_y_cells: i32,
    fade_ms: u64,
    audio_pad_ms: u64,
    audio_volume: f32,
}

#[derive(Debug, Clone)]
struct SplashScene {
    _id: String,
    _title: String,
    image_path: PathBuf,
    splatter_path: Option<PathBuf>,
    audio_path: PathBuf,
    bg_colour: style::Color,
    alpha_threshold: u8,
    logo_scale: f32,
    logo_align_x: Option<HorizontalAlign>,
    logo_align_y: Option<VerticalAlign>,
    logo_offset_x_cells: i32,
    logo_offset_y_cells: i32,
    blank_ms: u64,
    logo_fade_in_ms: u64,
    shake_delay_ms: u64,
    shake_ms: u64,
    shake_amplitude_cells: i16,
    shake_rotate_deg: f32,
    shake_punch_scale: f32,
    splatter_delay_ms: u64,
    splatter_reveal_ms: u64,
    splatter_drip_ms: u64,
    splatter_scale: f32,
    splatter_align_x: Option<HorizontalAlign>,
    splatter_align_y: Option<VerticalAlign>,
    splatter_offset_x_cells: i32,
    splatter_offset_y_cells: i32,
    fade_ms: u64,
    audio_pad_ms: u64,
    audio_volume: f32,
}

struct SplashVisuals {
    logo: image::RgbaImage,
    splatter: Option<image::RgbaImage>,
    preserve_layer_alignment: bool,
}

struct SvgRasterLayer {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    opacity: f32,
    image: image::RgbaImage,
}

#[derive(Clone, Copy)]
struct ImagePlacement {
    origin_x: u16,
    origin_y: u16,
    render_cols: u16,
    render_rows: u16,
}

#[derive(Clone, Copy)]
enum ImageColourMode {
    Source,
}

impl SplashScene {
    fn load(scene_path_override: Option<&Path>) -> io::Result<Self> {
        let scene_path = scene_path_override
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SPLASH_SCENE_PATH));
        let scene_raw = fs::read_to_string(&scene_path)?;
        let definition: SplashSceneDefinition = serde_yaml::from_str(&scene_raw)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let scene_dir = scene_path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid splash scene path")
        })?;

        Ok(Self {
            _id: definition.id,
            _title: definition.title,
            image_path: resolve_scene_asset(scene_dir, &definition.image),
            splatter_path: definition
                .splatter
                .as_deref()
                .map(|asset| resolve_scene_asset(scene_dir, asset)),
            audio_path: resolve_scene_asset(scene_dir, &definition.audio),
            bg_colour: convert_to_crossterm(Color::from(&definition.bg_colour)),
            alpha_threshold: definition.alpha_threshold,
            logo_scale: definition.logo_scale.max(0.1),
            logo_align_x: definition.logo_align_x,
            logo_align_y: definition.logo_align_y,
            logo_offset_x_cells: definition.logo_offset_x_cells,
            logo_offset_y_cells: definition.logo_offset_y_cells,
            blank_ms: definition.blank_ms,
            logo_fade_in_ms: definition.logo_fade_in_ms,
            shake_delay_ms: definition.shake_delay_ms,
            shake_ms: definition.shake_ms,
            shake_amplitude_cells: definition.shake_amplitude_cells.max(0),
            shake_rotate_deg: definition.shake_rotate_deg.abs(),
            shake_punch_scale: definition.shake_punch_scale.max(1.0),
            splatter_delay_ms: definition.splatter_delay_ms,
            splatter_reveal_ms: definition.splatter_reveal_ms,
            splatter_drip_ms: definition.splatter_drip_ms,
            splatter_scale: definition.splatter_scale.max(0.1),
            splatter_align_x: definition.splatter_align_x,
            splatter_align_y: definition.splatter_align_y,
            splatter_offset_x_cells: definition.splatter_offset_x_cells,
            splatter_offset_y_cells: definition.splatter_offset_y_cells,
            fade_ms: definition.fade_ms,
            audio_pad_ms: definition.audio_pad_ms,
            audio_volume: definition.audio_volume,
        })
    }

    fn total_ms(&self) -> u64 {
        splash_audio_duration_ms(&self.audio_path)
            .unwrap_or(0)
            .max(self.minimum_timeline_ms())
            .saturating_add(self.audio_pad_ms)
    }

    fn minimum_timeline_ms(&self) -> u64 {
        self.blank_ms
            .saturating_add(self.logo_fade_in_ms)
            .max(self.shake_delay_ms.saturating_add(self.shake_ms))
            .max(
                self.splatter_delay_ms
                    .saturating_add(self.splatter_reveal_ms.max(self.splatter_drip_ms)),
            )
            .saturating_add(self.fade_ms)
    }
}

/// Display the engine splash scene centered on the terminal.
///
/// Must be called after the alternate screen has been entered and the console
/// cleared. This blocks the calling thread while the splash is shown and plays
/// the splash audio in a background thread.
pub fn show_splash(target_bg: style::Color, scene_path_override: Option<&Path>) {
    let Some(splash_scene) = load_splash_scene(scene_path_override) else {
        return;
    };

    start_splash_audio(&splash_scene);
    if let Err(error) = try_show_splash(&splash_scene, target_bg) {
        crate::logging::warn("engine.splash", format!("splash display skipped: {error}"));
    }
}

/// Display splash content through the active render backend (SDL path).
///
/// This keeps splash rendering in the selected target window instead of using
/// terminal ANSI drawing.
#[cfg(feature = "sdl2")]
pub fn show_splash_on_output(
    output: &mut dyn OutputBackend,
    target_bg: style::Color,
    fit_size: (u16, u16),
    scene_path_override: Option<&Path>,
) {
    let Some(splash_scene) = load_splash_scene(scene_path_override) else {
        return;
    };

    start_splash_audio(&splash_scene);
    if let Err(error) = try_show_splash_on_output(output, &splash_scene, target_bg, fit_size) {
        crate::logging::warn(
            "engine.splash",
            format!("output splash display skipped: {error}"),
        );
    }
}

fn load_splash_scene(scene_path_override: Option<&Path>) -> Option<SplashScene> {
    match SplashScene::load(scene_path_override) {
        Ok(scene) => Some(scene),
        Err(primary_error) => {
            if scene_path_override.is_some() {
                crate::logging::warn(
                    "engine.splash",
                    format!(
                        "cannot load custom splash scene (falling back to engine default): {primary_error}"
                    ),
                );
                match SplashScene::load(None) {
                    Ok(scene) => return Some(scene),
                    Err(fallback_error) => {
                        crate::logging::warn(
                            "engine.splash",
                            format!("cannot load fallback splash scene: {fallback_error}"),
                        );
                        return None;
                    }
                }
            }
            crate::logging::warn(
                "engine.splash",
                format!("cannot load splash scene: {primary_error}"),
            );
            None
        }
    }
}

/// Plays the splash audio in a detached thread so playback continues beyond the
/// blocking splash hold/fade without leaking resources.
fn start_splash_audio(splash_scene: &SplashScene) {
    use rodio::{Decoder, OutputStream, Sink};

    let audio_path = splash_scene.audio_path.clone();
    let audio_volume = splash_scene.audio_volume.clamp(0.0, 1.0);

    if let Err(error) = std::thread::Builder::new()
        .name("splash-audio".into())
        .spawn(move || {
            let (_stream, handle) = match OutputStream::try_default() {
                Ok(pair) => pair,
                Err(error) => {
                    crate::logging::warn(
                        "engine.splash",
                        format!("cannot open audio device for splash jingle: {error}"),
                    );
                    return;
                }
            };

            let file = match fs::File::open(&audio_path) {
                Ok(file) => file,
                Err(error) => {
                    crate::logging::warn(
                        "engine.splash",
                        format!(
                            "cannot open splash audio '{}': {error}",
                            audio_path.display()
                        ),
                    );
                    return;
                }
            };

            let source = match Decoder::new(std::io::BufReader::new(file)) {
                Ok(source) => source,
                Err(error) => {
                    crate::logging::warn(
                        "engine.splash",
                        format!(
                            "cannot decode splash audio '{}': {error}",
                            audio_path.display()
                        ),
                    );
                    return;
                }
            };

            let sink = match Sink::try_new(&handle) {
                Ok(sink) => sink,
                Err(error) => {
                    crate::logging::warn(
                        "engine.splash",
                        format!("cannot create splash audio sink: {error}"),
                    );
                    return;
                }
            };

            sink.set_volume(audio_volume);
            sink.append(source);
            sink.sleep_until_end();
        })
    {
        crate::logging::warn(
            "engine.splash",
            format!("cannot spawn splash audio thread: {error}"),
        );
    }
}

fn try_show_splash(splash_scene: &SplashScene, target_bg: style::Color) -> io::Result<()> {
    let SplashVisuals {
        logo,
        splatter,
        preserve_layer_alignment,
    } = load_splash_visuals(splash_scene)?;
    let black_logo = build_flood_filled_silhouette(&logo);

    let (term_w, term_h) = terminal::size()?;
    let mut stdout = io::stdout();
    let logo_layout = placement_for_terminal(&logo, term_w, term_h)
        .map(|layout| scale_placement(layout, splash_scene.logo_scale, term_w, term_h))
        .map(|layout| {
            align_placement(
                layout,
                term_w,
                term_h,
                &splash_scene.logo_align_x,
                &splash_scene.logo_align_y,
                splash_scene.logo_offset_x_cells,
                splash_scene.logo_offset_y_cells,
            )
        });
    let splatter_layout = splatter.as_ref().and_then(|img| {
        if preserve_layer_alignment {
            logo_layout
        } else {
            logo_layout.map(|logo_layout| {
                let layout = scaled_placement(
                    logo_layout,
                    img,
                    splash_scene.splatter_scale,
                    term_w,
                    term_h,
                );
                align_placement(
                    layout,
                    term_w,
                    term_h,
                    &splash_scene.splatter_align_x,
                    &splash_scene.splatter_align_y,
                    splash_scene.splatter_offset_x_cells,
                    splash_scene.splatter_offset_y_cells,
                )
            })
        }
    });

    let total_ms = splash_scene.total_ms().max(1);
    let frame_interval = Duration::from_millis(16);
    let start = Instant::now();

    loop {
        let elapsed_ms = u64::try_from(start.elapsed().as_millis())
            .unwrap_or(u64::MAX)
            .min(total_ms);
        render_splash_frame(
            &mut stdout,
            term_w,
            term_h,
            &logo,
            &black_logo,
            logo_layout,
            splatter.as_ref(),
            splatter_layout,
            splash_scene,
            target_bg,
            elapsed_ms,
            total_ms,
        )?;
        stdout.flush()?;

        if elapsed_ms >= total_ms {
            break;
        }

        std::thread::sleep(frame_interval);
    }

    Ok(())
}

#[cfg(feature = "sdl2")]
fn try_show_splash_on_output(
    output: &mut dyn OutputBackend,
    splash_scene: &SplashScene,
    target_bg: style::Color,
    fit_size: (u16, u16),
) -> io::Result<()> {
    let SplashVisuals {
        logo,
        splatter,
        preserve_layer_alignment,
    } = load_splash_visuals(splash_scene)?;
    let black_logo = build_flood_filled_silhouette(&logo);

    let (term_w, term_h) = output.output_size();
    let term_w = term_w.max(1);
    let term_h = term_h.max(1);
    let fit_w = fit_size.0.max(1).min(term_w);
    let fit_h = fit_size.1.max(1).min(term_h);
    let logo_layout = placement_for_terminal(&logo, fit_w, fit_h)
        .map(|layout| scale_placement(layout, splash_scene.logo_scale, term_w, term_h))
        .map(|layout| scale_placement(layout, SDL_SPLASH_SCALE_FACTOR, term_w, term_h))
        .map(|layout| {
            align_placement(
                layout,
                term_w,
                term_h,
                &splash_scene.logo_align_x,
                &splash_scene.logo_align_y,
                splash_scene.logo_offset_x_cells,
                splash_scene.logo_offset_y_cells,
            )
        });
    let splatter_layout = splatter.as_ref().and_then(|img| {
        if preserve_layer_alignment {
            logo_layout
        } else {
            logo_layout.map(|logo_layout| {
                let layout = scaled_placement(
                    logo_layout,
                    img,
                    splash_scene.splatter_scale,
                    term_w,
                    term_h,
                );
                align_placement(
                    layout,
                    term_w,
                    term_h,
                    &splash_scene.splatter_align_x,
                    &splash_scene.splatter_align_y,
                    splash_scene.splatter_offset_x_cells,
                    splash_scene.splatter_offset_y_cells,
                )
            })
        }
    });

    let total_ms = splash_scene.total_ms().max(1);
    let frame_interval = Duration::from_millis(16);
    let start = Instant::now();
    let mut frame_buffer = Buffer::new(term_w, term_h);

    loop {
        let elapsed_ms = u64::try_from(start.elapsed().as_millis())
            .unwrap_or(u64::MAX)
            .min(total_ms);
        render_splash_frame_to_buffer(
            &mut frame_buffer,
            &logo,
            &black_logo,
            logo_layout,
            splatter.as_ref(),
            splatter_layout,
            splash_scene,
            target_bg,
            elapsed_ms,
            total_ms,
        )?;
        output.present_buffer(&frame_buffer);
        frame_buffer.swap();

        if elapsed_ms >= total_ms {
            break;
        }

        std::thread::sleep(frame_interval);
    }

    Ok(())
}

fn splash_audio_duration_ms(audio_path: &Path) -> Option<u64> {
    use rodio::{Decoder, Source};

    let file = fs::File::open(audio_path).ok()?;
    let source = Decoder::new(std::io::BufReader::new(file)).ok()?;
    let duration = source.total_duration()?;
    u64::try_from(duration.as_millis()).ok()
}

fn load_splash_visuals(splash_scene: &SplashScene) -> io::Result<SplashVisuals> {
    let is_svg = splash_scene
        .image_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));

    if is_svg {
        return load_svg_splash_visuals(&splash_scene.image_path);
    }

    let logo = load_bitmap_image(&splash_scene.image_path)?;
    let splatter = splash_scene
        .splatter_path
        .as_ref()
        .map(|path| load_bitmap_image(path))
        .transpose()?;
    Ok(SplashVisuals {
        logo,
        splatter,
        preserve_layer_alignment: false,
    })
}

fn load_bitmap_image(path: &Path) -> io::Result<image::RgbaImage> {
    let bytes = fs::read(path)?;
    load_from_memory(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
        .map(|img| img.to_rgba8())
}

fn load_svg_splash_visuals(path: &Path) -> io::Result<SplashVisuals> {
    let svg = fs::read_to_string(path)?;
    let doc = roxmltree::Document::parse(&svg)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let root = doc.root_element();
    let (canvas_w, canvas_h) = parse_svg_viewbox(&root)?;
    let mut layers = Vec::new();

    for node in root
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "image")
    {
        layers.push(parse_svg_raster_layer(&node)?);
    }

    if layers.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "splash svg contains no embedded image layers",
        ));
    }

    let mut rendered_layers = layers
        .into_iter()
        .map(|layer| render_svg_layer_to_canvas(layer, canvas_w, canvas_h))
        .collect::<Vec<_>>();
    let logo = rendered_layers
        .pop()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing splash logo layer"))?;
    let splatter = rendered_layers.into_iter().next();

    Ok(SplashVisuals {
        logo,
        splatter,
        preserve_layer_alignment: true,
    })
}

fn parse_svg_viewbox(root: &roxmltree::Node<'_, '_>) -> io::Result<(u32, u32)> {
    let view_box = root
        .attribute("viewBox")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "svg missing viewBox"))?;
    let mut parts = view_box.split_whitespace();
    let _min_x = parse_svg_number(parts.next().unwrap_or_default())?;
    let _min_y = parse_svg_number(parts.next().unwrap_or_default())?;
    let width = parse_svg_number(parts.next().unwrap_or_default())?;
    let height = parse_svg_number(parts.next().unwrap_or_default())?;

    if width <= 0.0 || height <= 0.0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "svg viewBox must have positive size",
        ));
    }

    Ok((width.round() as u32, height.round() as u32))
}

fn parse_svg_raster_layer(node: &roxmltree::Node<'_, '_>) -> io::Result<SvgRasterLayer> {
    let href = node
        .attribute("href")
        .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "svg image missing href"))?;
    let encoded = href.strip_prefix("data:image/png;base64,").ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "svg image is not embedded png")
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let image = load_from_memory(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .to_rgba8();

    Ok(SvgRasterLayer {
        x: parse_svg_number(node.attribute("x").unwrap_or("0"))?,
        y: parse_svg_number(node.attribute("y").unwrap_or("0"))?,
        width: parse_svg_number(node.attribute("width").unwrap_or("0"))?,
        height: parse_svg_number(node.attribute("height").unwrap_or("0"))?,
        opacity: parse_svg_number(node.attribute("opacity").unwrap_or("1"))?.clamp(0.0, 1.0),
        image,
    })
}

fn parse_svg_number(raw: &str) -> io::Result<f32> {
    let trimmed = raw.trim().trim_end_matches("px").trim_end_matches("in");
    trimmed
        .parse::<f32>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn render_svg_layer_to_canvas(
    layer: SvgRasterLayer,
    canvas_w: u32,
    canvas_h: u32,
) -> image::RgbaImage {
    let target_w = layer.width.max(1.0).round() as u32;
    let target_h = layer.height.max(1.0).round() as u32;
    let resized = imageops::resize(
        &layer.image,
        target_w,
        target_h,
        imageops::FilterType::Triangle,
    );
    let mut canvas = image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([0, 0, 0, 0]));
    let x = layer.x.round() as i64;
    let y = layer.y.round() as i64;

    if (layer.opacity - 1.0).abs() > f32::EPSILON {
        let mut adjusted = resized.clone();
        for pixel in adjusted.pixels_mut() {
            pixel[3] = ((pixel[3] as f32) * layer.opacity)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        imageops::overlay(&mut canvas, &adjusted, x, y);
    } else {
        imageops::overlay(&mut canvas, &resized, x, y);
    }

    canvas
}

fn build_flood_filled_silhouette(source: &image::RgbaImage) -> image::RgbaImage {
    let width = source.width();
    let height = source.height();
    if width == 0 || height == 0 {
        return source.clone();
    }

    let mut outside = vec![false; (width as usize) * (height as usize)];
    let mut queue = VecDeque::new();

    for x in 0..width {
        enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, x, 0);
        enqueue_outside_pixel(
            source,
            width,
            height,
            &mut outside,
            &mut queue,
            x,
            height.saturating_sub(1),
        );
    }

    for y in 0..height {
        enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, 0, y);
        enqueue_outside_pixel(
            source,
            width,
            height,
            &mut outside,
            &mut queue,
            width.saturating_sub(1),
            y,
        );
    }

    while let Some((x, y)) = queue.pop_front() {
        if x > 0 {
            enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, x - 1, y);
        }
        if x + 1 < width {
            enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, x + 1, y);
        }
        if y > 0 {
            enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, x, y - 1);
        }
        if y + 1 < height {
            enqueue_outside_pixel(source, width, height, &mut outside, &mut queue, x, y + 1);
        }
    }

    let mut silhouette = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 0]));
    for y in 0..height {
        for x in 0..width {
            let index = (y as usize) * (width as usize) + x as usize;
            if !outside[index] {
                silhouette.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
            }
        }
    }

    silhouette
}

fn enqueue_outside_pixel(
    source: &image::RgbaImage,
    width: u32,
    height: u32,
    outside: &mut [bool],
    queue: &mut VecDeque<(u32, u32)>,
    x: u32,
    y: u32,
) {
    if x >= width || y >= height || source.get_pixel(x, y)[3] > 0 {
        return;
    }

    let index = (y as usize) * (width as usize) + x as usize;
    if outside[index] {
        return;
    }

    outside[index] = true;
    queue.push_back((x, y));
}

fn render_splash_frame(
    stdout: &mut io::Stdout,
    term_w: u16,
    term_h: u16,
    logo: &image::RgbaImage,
    black_logo: &image::RgbaImage,
    logo_layout: Option<ImagePlacement>,
    splatter: Option<&image::RgbaImage>,
    splatter_layout: Option<ImagePlacement>,
    splash_scene: &SplashScene,
    target_bg: style::Color,
    elapsed_ms: u64,
    total_ms: u64,
) -> io::Result<()> {
    let fade_start_ms = total_ms.saturating_sub(splash_scene.fade_ms);
    let fade_t = if splash_scene.fade_ms == 0 || elapsed_ms <= fade_start_ms {
        0.0
    } else {
        (elapsed_ms.saturating_sub(fade_start_ms) as f32 / splash_scene.fade_ms as f32)
            .clamp(0.0, 1.0)
    };
    let bg = lerp_colour(splash_scene.bg_colour, target_bg, fade_t);
    fill_solid(stdout, term_w, term_h, bg)?;

    let (shake_x, shake_y, _shake_scale, rotation_deg) = shake_transform(
        elapsed_ms,
        splash_scene.shake_delay_ms,
        splash_scene.shake_ms,
        splash_scene.shake_amplitude_cells,
        splash_scene.shake_rotate_deg,
        splash_scene.shake_punch_scale,
    );

    if let (Some(splatter), Some(layout)) = (splatter, splatter_layout) {
        let reveal_t = phase_progress(
            elapsed_ms,
            splash_scene.splatter_delay_ms,
            splash_scene.splatter_reveal_ms,
        );
        let splatter_opacity = if splash_scene.splatter_reveal_ms == 0 {
            if elapsed_ms >= splash_scene.splatter_delay_ms {
                1.0 - fade_t
            } else {
                0.0
            }
        } else {
            let reveal = ease_out_cubic(reveal_t);
            (0.2 + reveal * 0.8) * (1.0 - fade_t)
        };
        let clip_y = if splash_scene.splatter_reveal_ms == 0 {
            1.0
        } else {
            ease_out_cubic(reveal_t)
        };
        let drip_t = phase_progress(
            elapsed_ms,
            splash_scene.splatter_delay_ms,
            splash_scene.splatter_drip_ms,
        );
        let drip_motion_t = ease_out_quad(drip_t);
        if splatter_opacity > 0.0 {
            let splatter_placement = ImagePlacement {
                origin_x: offset_cell(layout.origin_x, shake_x),
                origin_y: offset_cell(layout.origin_y, shake_y),
                ..layout
            };
            draw_image(
                stdout,
                splatter,
                splatter_placement,
                ImageColourMode::Source,
                bg,
                splash_scene.alpha_threshold,
                splatter_opacity,
                clip_y,
                rotation_deg,
                drip_motion_t,
            )?;
            draw_drip_tail(
                stdout,
                splatter,
                splatter_placement,
                term_h,
                splatter_opacity,
                drip_motion_t,
            )?;
        }
    }

    if let Some(layout) = logo_layout {
        let logo_opacity = phase_progress(
            elapsed_ms,
            splash_scene.blank_ms,
            splash_scene.logo_fade_in_ms,
        ) * (1.0 - fade_t);

        if logo_opacity <= 0.0 {
            return Ok(());
        }

        let (logo_image, logo_colour_mode) = if elapsed_ms >= splash_scene.splatter_delay_ms {
            (logo, ImageColourMode::Source)
        } else {
            (black_logo, ImageColourMode::Source)
        };

        draw_image(
            stdout,
            logo_image,
            ImagePlacement {
                origin_x: offset_cell(layout.origin_x, shake_x),
                origin_y: offset_cell(layout.origin_y, shake_y),
                ..layout
            },
            logo_colour_mode,
            bg,
            splash_scene.alpha_threshold,
            logo_opacity,
            1.0,
            rotation_deg,
            0.0,
        )?;
    }

    Ok(())
}

#[cfg(feature = "sdl2")]
fn render_splash_frame_to_buffer(
    target: &mut Buffer,
    logo: &image::RgbaImage,
    black_logo: &image::RgbaImage,
    logo_layout: Option<ImagePlacement>,
    splatter: Option<&image::RgbaImage>,
    splatter_layout: Option<ImagePlacement>,
    splash_scene: &SplashScene,
    target_bg: style::Color,
    elapsed_ms: u64,
    total_ms: u64,
) -> io::Result<()> {
    let fade_start_ms = total_ms.saturating_sub(splash_scene.fade_ms);
    let fade_t = if splash_scene.fade_ms == 0 || elapsed_ms <= fade_start_ms {
        0.0
    } else {
        (elapsed_ms.saturating_sub(fade_start_ms) as f32 / splash_scene.fade_ms as f32)
            .clamp(0.0, 1.0)
    };
    let bg = lerp_colour(splash_scene.bg_colour, target_bg, fade_t);
    fill_solid_buffer(target, bg);

    let (shake_x, shake_y, _shake_scale, rotation_deg) = shake_transform(
        elapsed_ms,
        splash_scene.shake_delay_ms,
        splash_scene.shake_ms,
        splash_scene.shake_amplitude_cells,
        splash_scene.shake_rotate_deg,
        splash_scene.shake_punch_scale,
    );

    if let (Some(splatter), Some(layout)) = (splatter, splatter_layout) {
        let reveal_t = phase_progress(
            elapsed_ms,
            splash_scene.splatter_delay_ms,
            splash_scene.splatter_reveal_ms,
        );
        let splatter_opacity = if splash_scene.splatter_reveal_ms == 0 {
            if elapsed_ms >= splash_scene.splatter_delay_ms {
                1.0 - fade_t
            } else {
                0.0
            }
        } else {
            let reveal = ease_out_cubic(reveal_t);
            (0.2 + reveal * 0.8) * (1.0 - fade_t)
        };
        let clip_y = if splash_scene.splatter_reveal_ms == 0 {
            1.0
        } else {
            ease_out_cubic(reveal_t)
        };
        let drip_t = phase_progress(
            elapsed_ms,
            splash_scene.splatter_delay_ms,
            splash_scene.splatter_drip_ms,
        );
        let drip_motion_t = ease_out_quad(drip_t);
        if splatter_opacity > 0.0 {
            let splatter_placement = ImagePlacement {
                origin_x: offset_cell(layout.origin_x, shake_x),
                origin_y: offset_cell(layout.origin_y, shake_y),
                ..layout
            };
            draw_image_to_buffer(
                target,
                splatter,
                splatter_placement,
                ImageColourMode::Source,
                bg,
                splash_scene.alpha_threshold,
                splatter_opacity,
                clip_y,
                rotation_deg,
                drip_motion_t,
            );
            draw_drip_tail_to_buffer(
                target,
                splatter,
                splatter_placement,
                splatter_opacity,
                drip_motion_t,
            )?;
        }
    }

    if let Some(layout) = logo_layout {
        let logo_opacity = phase_progress(
            elapsed_ms,
            splash_scene.blank_ms,
            splash_scene.logo_fade_in_ms,
        ) * (1.0 - fade_t);

        if logo_opacity <= 0.0 {
            return Ok(());
        }

        let (logo_image, logo_colour_mode) = if elapsed_ms >= splash_scene.splatter_delay_ms {
            (logo, ImageColourMode::Source)
        } else {
            (black_logo, ImageColourMode::Source)
        };

        draw_image_to_buffer(
            target,
            logo_image,
            ImagePlacement {
                origin_x: offset_cell(layout.origin_x, shake_x),
                origin_y: offset_cell(layout.origin_y, shake_y),
                ..layout
            },
            logo_colour_mode,
            bg,
            splash_scene.alpha_threshold,
            logo_opacity,
            1.0,
            rotation_deg,
            0.0,
        );
    }

    Ok(())
}

fn fill_solid(stdout: &mut io::Stdout, w: u16, h: u16, bg: style::Color) -> io::Result<()> {
    queue!(
        stdout,
        style::SetForegroundColor(bg),
        style::SetBackgroundColor(bg)
    )?;
    for y in 0..h {
        queue!(stdout, cursor::MoveTo(0, y))?;
        for _ in 0..w {
            queue!(stdout, style::Print(' '))?;
        }
    }
    Ok(())
}

#[cfg(feature = "sdl2")]
fn fill_solid_buffer(target: &mut Buffer, bg: style::Color) {
    target.fill(color_convert::from_crossterm(bg));
}

fn draw_image(
    stdout: &mut io::Stdout,
    img: &image::RgbaImage,
    placement: ImagePlacement,
    colour_mode: ImageColourMode,
    bg: style::Color,
    alpha_threshold: u8,
    opacity: f32,
    clip_y: f32,
    rotation_deg: f32,
    drip_t: f32,
) -> io::Result<()> {
    let virtual_h = placement.render_rows as u32 * 2;
    let bg_rgb = to_rgb(bg);

    for row in 0..placement.render_rows {
        for col in 0..placement.render_cols {
            let top = sample_transformed_clipped(
                img,
                col as u32,
                row as u32 * 2,
                placement.render_cols as u32,
                virtual_h,
                clip_y,
                rotation_deg,
                drip_t,
            );
            let bot = sample_transformed_clipped(
                img,
                col as u32,
                row as u32 * 2 + 1,
                placement.render_cols as u32,
                virtual_h,
                clip_y,
                rotation_deg,
                drip_t,
            );

            let (sym, cell_fg, cell_bg) = match render_halfblock_cell(
                top,
                bot,
                colour_mode,
                bg_rgb,
                alpha_threshold,
                opacity,
            ) {
                Some(v) => v,
                None => continue,
            };

            queue!(
                stdout,
                cursor::MoveTo(placement.origin_x + col, placement.origin_y + row),
                style::SetForegroundColor(cell_fg),
                style::SetBackgroundColor(cell_bg),
                style::Print(sym),
            )?;
        }
    }

    Ok(())
}

#[cfg(feature = "sdl2")]
fn draw_image_to_buffer(
    target: &mut Buffer,
    img: &image::RgbaImage,
    placement: ImagePlacement,
    colour_mode: ImageColourMode,
    bg: style::Color,
    alpha_threshold: u8,
    opacity: f32,
    clip_y: f32,
    rotation_deg: f32,
    drip_t: f32,
) {
    let virtual_h = placement.render_rows as u32 * 2;
    let bg_rgb = to_rgb(bg);

    for row in 0..placement.render_rows {
        for col in 0..placement.render_cols {
            let top = sample_transformed_clipped(
                img,
                col as u32,
                row as u32 * 2,
                placement.render_cols as u32,
                virtual_h,
                clip_y,
                rotation_deg,
                drip_t,
            );
            let bot = sample_transformed_clipped(
                img,
                col as u32,
                row as u32 * 2 + 1,
                placement.render_cols as u32,
                virtual_h,
                clip_y,
                rotation_deg,
                drip_t,
            );

            let (sym, cell_fg, cell_bg) = match render_halfblock_cell(
                top,
                bot,
                colour_mode,
                bg_rgb,
                alpha_threshold,
                opacity,
            ) {
                Some(v) => v,
                None => continue,
            };

            target.set(
                placement.origin_x.saturating_add(col),
                placement.origin_y.saturating_add(row),
                sym,
                color_convert::from_crossterm(cell_fg),
                color_convert::from_crossterm(cell_bg),
            );
        }
    }
}

fn render_halfblock_cell(
    top: [u8; 4],
    bot: [u8; 4],
    colour_mode: ImageColourMode,
    bg: (u8, u8, u8),
    alpha_threshold: u8,
    opacity: f32,
) -> Option<(char, style::Color, style::Color)> {
    let (top_on, top_rgb) = composite_pixel(top, colour_mode, bg, alpha_threshold, opacity);
    let (bot_on, bot_rgb) = composite_pixel(bot, colour_mode, bg, alpha_threshold, opacity);

    match (top_on, bot_on) {
        (false, false) => None,
        (true, false) => Some(('▀', rgb(top_rgb), rgb(bg))),
        (false, true) => Some(('▄', rgb(bot_rgb), rgb(bg))),
        (true, true) => Some(('▀', rgb(top_rgb), rgb(bot_rgb))),
    }
}

fn composite_pixel(
    pixel: [u8; 4],
    colour_mode: ImageColourMode,
    bg: (u8, u8, u8),
    alpha_threshold: u8,
    opacity: f32,
) -> (bool, (u8, u8, u8)) {
    let a = (pixel[3] as f32 * opacity).clamp(0.0, 255.0);
    if a < alpha_threshold as f32 {
        return (false, bg);
    }
    let source = (pixel[0], pixel[1], pixel[2]);
    let (fg, alpha) = match colour_mode {
        ImageColourMode::Source => (source, (a / 255.0).clamp(0.0, 1.0)),
    };
    (true, blend(bg, fg, alpha))
}

fn blend(bg: (u8, u8, u8), fg: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    let inv = 1.0 - alpha;
    (
        (bg.0 as f32 * inv + fg.0 as f32 * alpha)
            .round()
            .clamp(0.0, 255.0) as u8,
        (bg.1 as f32 * inv + fg.1 as f32 * alpha)
            .round()
            .clamp(0.0, 255.0) as u8,
        (bg.2 as f32 * inv + fg.2 as f32 * alpha)
            .round()
            .clamp(0.0, 255.0) as u8,
    )
}

fn lerp_colour(from: style::Color, to: style::Color, t: f32) -> style::Color {
    let t = t.clamp(0.0, 1.0);
    let a = to_rgb(from);
    let b = to_rgb(to);
    rgb(blend(a, b, t))
}

fn rgb((r, g, b): (u8, u8, u8)) -> style::Color {
    style::Color::Rgb { r, g, b }
}

fn to_rgb(c: style::Color) -> (u8, u8, u8) {
    use style::Color;
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (85, 85, 85),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        Color::White => (255, 255, 255),
        Color::Grey => (192, 192, 192),
        Color::Reset => (0, 0, 0),
        Color::AnsiValue(_) => (0, 0, 0),
    }
}

/// Compute (render_cols, render_rows) that fit the logo into the terminal,
/// preserving aspect ratio and leaving a 1-cell margin.
fn fit_logo(img_w: u32, img_h: u32, term_w: u16, term_h: u16) -> (u16, u16) {
    let max_cols = term_w.saturating_sub(2) as u32;
    let max_rows = term_h.saturating_sub(2) as u32;
    if max_cols == 0 || max_rows == 0 || img_w == 0 || img_h == 0 {
        return (0, 0);
    }
    let pixel_rows = img_h;
    let cols_by_w = max_cols;
    let rows_by_w = (pixel_rows * cols_by_w / img_w / 2).max(1);
    let rows_by_h = max_rows;
    let cols_by_h = (img_w * rows_by_h * 2 / pixel_rows).max(1);

    let (cols, rows) = if rows_by_w <= max_rows {
        (cols_by_w, rows_by_w)
    } else {
        (cols_by_h.min(max_cols), rows_by_h)
    };
    (
        cols.min(u16::MAX as u32) as u16,
        rows.min(u16::MAX as u32) as u16,
    )
}

fn placement_for_terminal(
    img: &image::RgbaImage,
    term_w: u16,
    term_h: u16,
) -> Option<ImagePlacement> {
    let (render_cols, render_rows) = fit_logo(img.width(), img.height(), term_w, term_h);
    if render_cols == 0 || render_rows == 0 {
        return None;
    }

    Some(ImagePlacement {
        origin_x: term_w.saturating_sub(render_cols) / 2,
        origin_y: term_h.saturating_sub(render_rows) / 2,
        render_cols,
        render_rows,
    })
}

fn scaled_placement(
    base: ImagePlacement,
    img: &image::RgbaImage,
    scale: f32,
    term_w: u16,
    term_h: u16,
) -> ImagePlacement {
    let scaled_cols = ((base.render_cols as f32 * scale).round()).clamp(1.0, term_w as f32) as u16;
    let scaled_rows = ((base.render_rows as f32 * scale).round()).clamp(1.0, term_h as f32) as u16;
    let (render_cols, render_rows) = fit_logo(
        img.width(),
        img.height(),
        scaled_cols.saturating_add(2),
        scaled_rows.saturating_add(2),
    );
    let render_cols = render_cols.max(1);
    let render_rows = render_rows.max(1);

    ImagePlacement {
        origin_x: base.origin_x + base.render_cols.saturating_sub(render_cols) / 2,
        origin_y: base.origin_y + base.render_rows.saturating_sub(render_rows) / 2,
        render_cols,
        render_rows,
    }
}

fn scale_placement(base: ImagePlacement, scale: f32, term_w: u16, term_h: u16) -> ImagePlacement {
    let render_cols = ((base.render_cols as f32 * scale).round()).clamp(1.0, term_w as f32) as u16;
    let render_rows = ((base.render_rows as f32 * scale).round()).clamp(1.0, term_h as f32) as u16;

    ImagePlacement {
        origin_x: base.origin_x + base.render_cols.saturating_sub(render_cols) / 2,
        origin_y: base.origin_y + base.render_rows.saturating_sub(render_rows) / 2,
        render_cols,
        render_rows,
    }
}

fn sample_transformed_clipped(
    img: &image::RgbaImage,
    col: u32,
    pixel_row: u32,
    cols: u32,
    pixel_rows: u32,
    clip_y: f32,
    rotation_deg: f32,
    drip_t: f32,
) -> [u8; 4] {
    if clip_y <= 0.0 {
        return [0, 0, 0, 0];
    }
    let clip_limit = ((pixel_rows as f32) * clip_y.clamp(0.0, 1.0)).ceil() as u32;
    if pixel_row >= clip_limit {
        return [0, 0, 0, 0];
    }

    let cols = cols.max(1);
    let pixel_rows = pixel_rows.max(1);
    let u = (col as f32 + 0.5) / cols as f32;
    let v = (pixel_row as f32 + 0.5) / pixel_rows as f32;
    let centered_x = u - 0.5;
    let centered_y = v - 0.5;
    let theta = -rotation_deg.to_radians();
    let (sin_t, cos_t) = theta.sin_cos();
    let src_x = centered_x * cos_t - centered_y * sin_t + 0.5;
    let src_y = centered_x * sin_t + centered_y * cos_t + 0.5;

    if !(0.0..=1.0).contains(&src_x) || !(0.0..=1.0).contains(&src_y) {
        return [0, 0, 0, 0];
    }

    if drip_t > 0.0 && !passes_drip_mask(src_x, src_y, drip_t) {
        return [0, 0, 0, 0];
    }

    let px = (src_x * img.width() as f32).floor() as u32;
    let py = (src_y * img.height() as f32).floor() as u32;
    let px = px.min(img.width().saturating_sub(1));
    let py = py.min(img.height().saturating_sub(1));
    img.get_pixel(px, py).0
}

fn resolve_scene_asset(scene_dir: &Path, asset_ref: &str) -> PathBuf {
    let asset_path = Path::new(asset_ref);
    if asset_path.is_absolute() {
        asset_path.to_path_buf()
    } else {
        scene_dir.join(asset_path)
    }
}

fn align_placement(
    placement: ImagePlacement,
    term_w: u16,
    term_h: u16,
    align_x: &Option<HorizontalAlign>,
    align_y: &Option<VerticalAlign>,
    offset_x: i32,
    offset_y: i32,
) -> ImagePlacement {
    let max_x = i32::from(term_w.saturating_sub(placement.render_cols));
    let max_y = i32::from(term_h.saturating_sub(placement.render_rows));
    let origin_x =
        resolve_align_x(offset_x, align_x, term_w, placement.render_cols).clamp(0, max_x) as u16;
    let origin_y =
        resolve_align_y(offset_y, align_y, term_h, placement.render_rows).clamp(0, max_y) as u16;

    ImagePlacement {
        origin_x,
        origin_y,
        ..placement
    }
}

fn resolve_align_x(
    offset_x: i32,
    align_x: &Option<HorizontalAlign>,
    area_w: u16,
    sprite_w: u16,
) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) | None => 0i32,
        Some(HorizontalAlign::Center) => (area_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => area_w.saturating_sub(sprite_w) as i32,
    };
    origin.saturating_add(offset_x)
}

fn resolve_align_y(
    offset_y: i32,
    align_y: &Option<VerticalAlign>,
    area_h: u16,
    sprite_h: u16,
) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) | None => 0i32,
        Some(VerticalAlign::Center) => (area_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => area_h.saturating_sub(sprite_h) as i32,
    };
    origin.saturating_add(offset_y)
}

fn offset_cell(value: u16, delta: i16) -> u16 {
    if delta >= 0 {
        value.saturating_add(delta as u16)
    } else {
        value.saturating_sub(delta.unsigned_abs())
    }
}

fn phase_progress(elapsed_ms: u64, delay_ms: u64, duration_ms: u64) -> f32 {
    if elapsed_ms < delay_ms {
        0.0
    } else if duration_ms == 0 {
        1.0
    } else {
        (elapsed_ms.saturating_sub(delay_ms) as f32 / duration_ms as f32).clamp(0.0, 1.0)
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let inv = 1.0 - t.clamp(0.0, 1.0);
    1.0 - inv * inv * inv
}

fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

fn shake_transform(
    elapsed_ms: u64,
    shake_delay_ms: u64,
    shake_ms: u64,
    amplitude_cells: i16,
    rotate_deg: f32,
    punch_scale: f32,
) -> (i16, i16, f32, f32) {
    if shake_ms == 0 || elapsed_ms < shake_delay_ms || amplitude_cells <= 0 {
        return (0, 0, 1.0, 0.0);
    }

    let local_ms = elapsed_ms.saturating_sub(shake_delay_ms);
    if local_ms >= shake_ms {
        return (0, 0, 1.0, 0.0);
    }

    let progress = (local_ms as f32 / shake_ms as f32).clamp(0.0, 1.0);
    let decay = 1.0 - ease_out_cubic(progress);
    let amplitude = amplitude_cells as f32 * decay;
    let slice = u32::try_from(local_ms / 16).unwrap_or(u32::MAX);
    let x = signed_unit_hash(slice ^ 0x9e37_79b9) * amplitude;
    let y = signed_unit_hash(slice ^ 0x7f4a_7c15) * amplitude;
    let rotation = signed_unit_hash(slice ^ 0x85eb_ca6b) * rotate_deg * decay;
    let scale = if slice == 0 {
        1.0 + ((punch_scale - 1.0) * 0.5)
    } else if slice == 1 {
        punch_scale
    } else {
        1.0 + (punch_scale - 1.0) * (1.0 - progress) * 0.35
    };

    (x.round() as i16, y.round() as i16, scale, rotation)
}

fn signed_unit_hash(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    x = (x >> ((x >> 28) + 4)) ^ x;
    x = x.wrapping_mul(277_803_737);
    let normalized = ((x >> 9) as f32 / ((1u32 << 23) as f32)).fract();
    normalized * 2.0 - 1.0
}

fn passes_drip_mask(u: f32, v: f32, drip_t: f32) -> bool {
    let t = drip_t.clamp(0.0, 1.0);
    if v <= 0.68 {
        return true;
    }

    let mut drip_edge = 0.68f32;
    for idx in 0..5u32 {
        let center = 0.16 + idx as f32 * 0.17 + signed_unit_hash(0x51f2_0000 + idx) * 0.03;
        let width = 0.05 + idx as f32 * 0.008;
        let distance = ((u - center).abs() / width.max(0.02)).clamp(0.0, 1.0);
        let profile = 1.0 - distance * distance;
        let length = (0.12 + idx as f32 * 0.05) * ease_out_quad(t) * profile.max(0.0);
        drip_edge = drip_edge.max(0.68 + length);
    }

    v <= drip_edge.min(1.0)
}

fn draw_drip_tail(
    stdout: &mut io::Stdout,
    img: &image::RgbaImage,
    placement: ImagePlacement,
    term_h: u16,
    opacity: f32,
    drip_t: f32,
) -> io::Result<()> {
    let max_tail_rows =
        term_h.saturating_sub(placement.origin_y.saturating_add(placement.render_rows));
    if max_tail_rows == 0 || drip_t <= 0.0 || placement.render_cols == 0 {
        return Ok(());
    }

    let mut buffer = transparent_buffer(placement.render_cols, max_tail_rows.saturating_add(1));
    for col in 0..placement.render_cols {
        let u = (col as f32 + 0.5) / placement.render_cols as f32;
        let Some(paint) = sample_drip_colour(img, u) else {
            continue;
        };
        let paint_colour = Color::Rgb {
            r: paint.0,
            g: paint.1,
            b: paint.2,
        };
        buffer.set(col, 0, ' ', Color::Reset, paint_colour);
        if max_tail_rows > 0 {
            buffer.set(col, 1.min(max_tail_rows), ' ', Color::Reset, paint_colour);
        }
    }

    let params = crate::scene::EffectParams {
        colour: None,
        intensity: Some(1.0),
        speed: Some(1.0),
        thickness: Some(1.25),
        alpha: Some(opacity.clamp(0.0, 1.0)),
        distortion: Some(0.3),
        brightness: Some(0.18),
        falloff: Some(1.35),
        ..crate::scene::EffectParams::default()
    };
    shared_dispatcher().apply(
        "paint-splatter",
        drip_t,
        &params,
        Region::full(&buffer),
        &mut buffer,
    );
    render_buffer_overlay(
        stdout,
        &buffer,
        placement.origin_x,
        placement.origin_y + placement.render_rows.saturating_sub(1),
    )?;
    Ok(())
}

#[cfg(feature = "sdl2")]
fn draw_drip_tail_to_buffer(
    target: &mut Buffer,
    img: &image::RgbaImage,
    placement: ImagePlacement,
    opacity: f32,
    drip_t: f32,
) -> io::Result<()> {
    let max_tail_rows = target
        .height
        .saturating_sub(placement.origin_y.saturating_add(placement.render_rows));
    if max_tail_rows == 0 || drip_t <= 0.0 || placement.render_cols == 0 {
        return Ok(());
    }

    let mut buffer = transparent_buffer(placement.render_cols, max_tail_rows.saturating_add(1));
    for col in 0..placement.render_cols {
        let u = (col as f32 + 0.5) / placement.render_cols as f32;
        let Some(paint) = sample_drip_colour(img, u) else {
            continue;
        };
        let paint_colour = Color::Rgb {
            r: paint.0,
            g: paint.1,
            b: paint.2,
        };
        buffer.set(col, 0, ' ', Color::Reset, paint_colour);
        if max_tail_rows > 0 {
            buffer.set(col, 1.min(max_tail_rows), ' ', Color::Reset, paint_colour);
        }
    }

    let params = crate::scene::EffectParams {
        colour: None,
        intensity: Some(1.0),
        speed: Some(1.0),
        thickness: Some(1.25),
        alpha: Some(opacity.clamp(0.0, 1.0)),
        distortion: Some(0.3),
        brightness: Some(0.18),
        falloff: Some(1.35),
        ..crate::scene::EffectParams::default()
    };
    shared_dispatcher().apply(
        "paint-splatter",
        drip_t,
        &params,
        Region::full(&buffer),
        &mut buffer,
    );
    blit_buffer_overlay(
        target,
        &buffer,
        placement.origin_x,
        placement.origin_y + placement.render_rows.saturating_sub(1),
    );
    Ok(())
}

fn transparent_buffer(width: u16, height: u16) -> Buffer {
    let mut buffer = Buffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            buffer.set(x, y, ' ', Color::Reset, Color::Reset);
        }
    }
    buffer
}

#[cfg(feature = "sdl2")]
fn blit_buffer_overlay(target: &mut Buffer, overlay: &Buffer, origin_x: u16, origin_y: u16) {
    for y in 0..overlay.height {
        for x in 0..overlay.width {
            let Some(cell) = overlay.get(x, y).copied() else {
                continue;
            };
            if cell.symbol == ' ' && cell.fg == Color::Reset && cell.bg == Color::Reset {
                continue;
            }
            target.set(
                origin_x.saturating_add(x),
                origin_y.saturating_add(y),
                cell.symbol,
                cell.fg,
                cell.bg,
            );
        }
    }
}

fn render_buffer_overlay(
    stdout: &mut io::Stdout,
    buffer: &Buffer,
    origin_x: u16,
    origin_y: u16,
) -> io::Result<()> {
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let Some(cell) = buffer.get(x, y).copied() else {
                continue;
            };
            if cell.symbol == ' ' && cell.fg == Color::Reset && cell.bg == Color::Reset {
                continue;
            }
            queue!(
                stdout,
                cursor::MoveTo(origin_x + x, origin_y + y),
                style::SetForegroundColor(convert_to_crossterm(cell.fg)),
                style::SetBackgroundColor(convert_to_crossterm(cell.bg)),
                style::Print(cell.symbol),
            )?;
        }
    }
    Ok(())
}

fn sample_drip_colour(img: &image::RgbaImage, u: f32) -> Option<(u8, u8, u8)> {
    let x = ((u.clamp(0.0, 1.0) * img.width().saturating_sub(1) as f32).round() as u32)
        .min(img.width().saturating_sub(1));
    for y in (0..img.height()).rev() {
        let pixel = img.get_pixel(x, y).0;
        if pixel[3] >= 24 {
            return Some((pixel[0], pixel[1], pixel[2]));
        }
    }
    None
}

/// Convert engine_core::color::Color to crossterm::style::Color.
fn convert_to_crossterm(c: Color) -> style::Color {
    color_convert::to_crossterm(c)
}

#[cfg(test)]
mod tests {
    use super::config_from_manifest;
    use std::path::Path;

    #[test]
    fn splash_config_defaults_when_missing() {
        let manifest: serde_yaml::Value =
            serde_yaml::from_str("name: test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\n")
                .expect("manifest parses");
        let cfg = config_from_manifest(Path::new("mods/demo"), &manifest);
        assert!(cfg.enabled);
        assert!(cfg.scene_path.is_none());
    }

    #[test]
    fn splash_config_parses_enabled_and_absolute_scene_path() {
        let manifest: serde_yaml::Value = serde_yaml::from_str(
            "name: test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nsplash:\n  enabled: false\n  scene: /scenes/splash/scene.yml\n",
        )
        .expect("manifest parses");
        let cfg = config_from_manifest(Path::new("mods/demo"), &manifest);
        assert!(!cfg.enabled);
        assert_eq!(
            cfg.scene_path.as_deref(),
            Some(Path::new("mods/demo/scenes/splash/scene.yml"))
        );
    }

    #[test]
    fn splash_config_parses_relative_scene_path_alias() {
        let manifest: serde_yaml::Value = serde_yaml::from_str(
            "name: test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nsplash:\n  scene-path: config/splash.yml\n",
        )
        .expect("manifest parses");
        let cfg = config_from_manifest(Path::new("mods/demo"), &manifest);
        assert!(cfg.enabled);
        assert_eq!(
            cfg.scene_path.as_deref(),
            Some(Path::new("mods/demo/config/splash.yml"))
        );
    }
}
