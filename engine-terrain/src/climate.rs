//! Moisture and temperature fields.

use crate::grid::cell_to_xyz;
use crate::noise::fbm;

/// Build moisture and temperature grids (values 0.0–1.0).
///
/// - Moisture: latitude-gradient pattern (ITCZ + mid-latitude bands) blended with
///   noise-driven regional variation. Higher values = wetter.
/// - Temperature: latitude-based gradient (hot equator, cold poles) scaled by
///   `ice_cap_strength`, minus an elevation lapse rate controlled by `lapse_rate`.
pub fn build(
    elevation: &[f32],
    width: usize,
    height: usize,
    seed: u64,
    moisture_scale: f64,
    ice_cap_strength: f64,
    lapse_rate: f64,
    rain_shadow: f64,
) -> (Vec<f32>, Vec<f32>) {
    let total = width * height;
    let mut moisture = vec![0.0f32; total];
    let mut temperature = vec![0.0f32; total];

    let ice = ice_cap_strength.clamp(0.0, 3.0) as f32;
    let lapse = lapse_rate.clamp(0.0, 1.0) as f32;
    let shadow = rain_shadow.clamp(0.0, 1.0) as f32;

    for y in 0..height {
        let lat_norm = (y as f32 + 0.5) / height as f32;
        let lat_rad = lat_norm * std::f32::consts::PI;
        let from_equator = (lat_rad - std::f32::consts::FRAC_PI_2).abs() / std::f32::consts::FRAC_PI_2;

        // Temperature: equator hot, poles cold — strength scaled by ice_cap_strength
        let base_temp = (1.0 - from_equator * ice).clamp(0.0, 1.0);

        let lat_moisture = {
            let t = from_equator;
            let itcz   =       (-((t - 0.00) * 4.0).powi(2)).exp();
            let midlat = 0.5 * (-((t - 0.45) * 5.0).powi(2)).exp();
            (itcz + midlat).min(1.0)
        };

        for x in 0..width {
            let idx = y * width + x;
            let elev = elevation[idx];

            let (cx, cy, cz) = cell_to_xyz(x, y, width, height);
            let ms = moisture_scale;
            let noise_m = fbm(cx * ms, cy * ms, cz * ms, 4, seed + 1000) as f32;

            let m = lat_moisture * 0.55 + noise_m * 0.45;

            // Rain shadow: above sea level reduces moisture (parameterized)
            let shadow_factor = if elev > 0.5 { ((elev - 0.5) * 2.0) * shadow } else { 0.0 };
            moisture[idx] = (m - shadow_factor).clamp(0.0, 1.0);

            // Temperature: base − elevation lapse (parameterized)
            let lapse_factor = if elev > 0.5 { (elev - 0.5) * 2.0 * lapse } else { 0.0 };
            temperature[idx] = (base_temp - lapse_factor).clamp(0.0, 1.0);
        }
    }

    (moisture, temperature)
}
