//! Moisture and temperature fields.

use crate::grid::cell_to_xyz;
use crate::noise::fbm;

/// Build moisture and temperature grids (values 0.0–1.0).
///
/// - Moisture: latitude-gradient pattern (ITCZ + mid-latitude bands) blended with
///   noise-driven regional variation. Higher values = wetter.
/// - Temperature: latitude-based gradient (hot equator, cold poles) minus an
///   elevation lapse rate (high ground is colder).
pub fn build(
    elevation: &[f32],
    width: usize,
    height: usize,
    seed: u64,
    moisture_scale: f64,
) -> (Vec<f32>, Vec<f32>) {
    let total = width * height;
    let mut moisture = vec![0.0f32; total];
    let mut temperature = vec![0.0f32; total];

    for y in 0..height {
        // lat_norm: 0 = north pole, 1 = south pole (consistent with grid.rs)
        let lat_norm = (y as f32 + 0.5) / height as f32;
        let lat_rad = lat_norm * std::f32::consts::PI;
        // Distance from equator: 0 = equator, 1 = pole
        let from_equator = (lat_rad - std::f32::consts::FRAC_PI_2).abs() / std::f32::consts::FRAC_PI_2;

        // Base temperature: hot equator, cold poles
        let base_temp = 1.0 - from_equator;

        // Latitude moisture pattern: peaks at equator (ITCZ) and mid-latitudes (~0.45 from equator)
        let lat_moisture = {
            let t = from_equator;
            let itcz   =       (-((t - 0.00) * 4.0).powi(2)).exp();
            let midlat = 0.5 * (-((t - 0.45) * 5.0).powi(2)).exp();
            (itcz + midlat).min(1.0)
        };

        for x in 0..width {
            let idx = y * width + x;
            let elev = elevation[idx];

            // Regional moisture variation: noise-driven so different continents differ
            let (cx, cy, cz) = cell_to_xyz(x, y, width, height);
            let ms = moisture_scale;
            let noise_m = fbm(cx * ms, cy * ms, cz * ms, 4, seed + 1000) as f32;

            // Blend: 55% latitude pattern, 45% regional noise
            let m = lat_moisture * 0.55 + noise_m * 0.45;

            // Rain shadow: above sea level reduces moisture
            let shadow = if elev > 0.5 { ((elev - 0.5) * 2.0) * 0.35 } else { 0.0 };
            moisture[idx] = (m - shadow).clamp(0.0, 1.0);

            // Temperature: latitude base − elevation lapse
            let lapse = if elev > 0.5 { (elev - 0.5) * 2.0 * 0.6 } else { 0.0 };
            temperature[idx] = (base_temp - lapse).clamp(0.0, 1.0);
        }
    }

    (moisture, temperature)
}
