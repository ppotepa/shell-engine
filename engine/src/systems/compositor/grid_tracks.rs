#[derive(Clone, Copy)]
pub(crate) enum TrackSpec {
    Auto,
    Fr(u16),
    Fixed(u16),
}

pub(crate) fn parse_track_spec(input: &str) -> TrackSpec {
    let spec = input.trim().to_ascii_lowercase();
    if spec.is_empty() || spec == "auto" {
        return TrackSpec::Auto;
    }
    if let Some(weight) = spec.strip_suffix("fr") {
        let w = weight.trim().parse::<u16>().unwrap_or(1).max(1);
        return TrackSpec::Fr(w);
    }
    if let Ok(px) = spec.parse::<u16>() {
        return TrackSpec::Fixed(px.max(1));
    }
    TrackSpec::Auto
}

pub(crate) fn resolve_track_sizes(
    specs: &[TrackSpec],
    container: u16,
    gap: u16,
    auto_reqs: &[(usize, u16)],
) -> Vec<u16> {
    if specs.is_empty() {
        return vec![container.max(1)];
    }

    let mut sizes = vec![0u16; specs.len()];
    for (idx, spec) in specs.iter().enumerate() {
        if let TrackSpec::Fixed(px) = spec {
            sizes[idx] = *px;
        }
    }

    for (idx, pref) in auto_reqs {
        if *idx >= specs.len() {
            continue;
        }
        if matches!(specs[*idx], TrackSpec::Auto) {
            sizes[*idx] = sizes[*idx].max(*pref);
        }
    }

    let gap_total = gap.saturating_mul((specs.len().saturating_sub(1)) as u16);
    let used = sizes
        .iter()
        .copied()
        .fold(0u16, u16::saturating_add)
        .saturating_add(gap_total);
    let mut remaining = container.saturating_sub(used);

    let fr_total: u32 = specs
        .iter()
        .map(|s| match s {
            TrackSpec::Fr(w) => *w as u32,
            _ => 0,
        })
        .sum();

    if fr_total > 0 && remaining > 0 {
        let mut distributed = 0u16;
        let mut fr_indices = Vec::new();
        for (idx, spec) in specs.iter().enumerate() {
            if let TrackSpec::Fr(weight) = spec {
                fr_indices.push((idx, *weight));
                let share = ((remaining as u32) * (*weight as u32) / fr_total) as u16;
                sizes[idx] = share;
                distributed = distributed.saturating_add(share);
            }
        }

        remaining = remaining.saturating_sub(distributed);
        let mut i = 0usize;
        while remaining > 0 && !fr_indices.is_empty() {
            let (idx, _) = fr_indices[i % fr_indices.len()];
            sizes[idx] = sizes[idx].saturating_add(1);
            remaining = remaining.saturating_sub(1);
            i += 1;
        }
    }

    sizes
}

pub(crate) fn track_start(sizes: &[u16], gap: u16, track_idx: usize) -> u16 {
    let mut pos = 0u16;
    for (i, size) in sizes.iter().enumerate() {
        if i >= track_idx {
            break;
        }
        pos = pos.saturating_add(*size);
        pos = pos.saturating_add(gap);
    }
    pos
}

pub(crate) fn span_size(sizes: &[u16], gap: u16, start_idx: usize, span: usize) -> u16 {
    let end = (start_idx + span).min(sizes.len());
    if start_idx >= end {
        return 1;
    }
    let mut size = 0u16;
    for (i, s) in sizes.iter().enumerate().take(end).skip(start_idx) {
        size = size.saturating_add(*s);
        if i + 1 < end {
            size = size.saturating_add(gap);
        }
    }
    size
}

#[cfg(test)]
mod tests {
    use super::{parse_track_spec, resolve_track_sizes, span_size, track_start, TrackSpec};

    #[test]
    fn parses_track_specs() {
        assert!(matches!(parse_track_spec("auto"), TrackSpec::Auto));
        assert!(matches!(parse_track_spec("2fr"), TrackSpec::Fr(2)));
        assert!(matches!(parse_track_spec("12"), TrackSpec::Fixed(12)));
    }

    #[test]
    fn distributes_fr_space() {
        let specs = vec![TrackSpec::Fr(1), TrackSpec::Fr(2)];
        let sizes = resolve_track_sizes(&specs, 30, 0, &[]);
        assert_eq!(sizes.iter().sum::<u16>(), 30);
        assert!(sizes[1] >= sizes[0]);
    }

    #[test]
    fn computes_track_positions_and_span() {
        let sizes = vec![10, 20, 5];
        assert_eq!(track_start(&sizes, 1, 0), 0);
        assert_eq!(track_start(&sizes, 1, 1), 11);
        assert_eq!(span_size(&sizes, 1, 0, 2), 31);
    }
}
