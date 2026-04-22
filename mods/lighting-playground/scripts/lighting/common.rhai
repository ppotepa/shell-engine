fn clamp01(v) {
    let out = v;
    if out < 0.0 { out = 0.0; }
    if out > 1.0 { out = 1.0; }
    out
}
