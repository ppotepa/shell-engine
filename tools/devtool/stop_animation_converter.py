#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path
from typing import Iterable, List

from PIL import Image, ImageEnhance, ImageOps


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            'Convert a stop-motion frame sequence into a retro GIF with '
            'stable global palette, mild contrast extraction, and optional dithering.'
        )
    )
    parser.add_argument('inputs', nargs='+', help='Input frame files in playback order.')
    parser.add_argument('-o', '--output', required=True, help='Output GIF path.')
    parser.add_argument('--fps', type=float, default=12.0, help='Playback FPS. Default: 12.')
    parser.add_argument('--colors', type=int, default=256, help='Palette size. Default: 256.')
    parser.add_argument('--contrast', type=float, default=1.0, help='Contrast multiplier. Default: 1.0.')
    parser.add_argument('--autocontrast-cutoff', type=float, default=0.0, help='Auto-contrast cutoff percent per side. Default: 0.0.')
    parser.add_argument('--sharpness', type=float, default=1.0, help='Sharpness multiplier. Default: 1.0.')
    parser.add_argument('--dither', action='store_true', help='Enable Floyd-Steinberg dithering.')
    parser.add_argument('--loop', type=int, default=0, help='GIF loop count. Default: 0 (infinite).')
    return parser.parse_args()


def load_frames(paths: Iterable[str]) -> List[Image.Image]:
    frames: List[Image.Image] = []
    expected_size = None
    for raw in paths:
        path = Path(raw)
        if not path.exists():
            raise SystemExit(f'missing input frame: {path}')
        image = Image.open(path).convert('RGBA')
        if expected_size is None:
            expected_size = image.size
        elif image.size != expected_size:
            raise SystemExit(
                f'inconsistent frame size: {path} has {image.size}, expected {expected_size}'
            )
        frames.append(image)
    if not frames:
        raise SystemExit('no frames loaded')
    return frames


def preprocess(frame: Image.Image, contrast: float, cutoff: float, sharpness: float) -> Image.Image:
    processed = frame.convert('RGB')
    if cutoff > 0:
        processed = ImageOps.autocontrast(processed, cutoff=cutoff)
    if contrast != 1.0:
        processed = ImageEnhance.Contrast(processed).enhance(contrast)
    if sharpness != 1.0:
        processed = ImageEnhance.Sharpness(processed).enhance(sharpness)
    return processed


def build_global_palette(frames: List[Image.Image], colors: int, dither: int) -> Image.Image:
    width, height = frames[0].size
    strip = Image.new('RGB', (width, height * len(frames)))
    for index, frame in enumerate(frames):
        strip.paste(frame.convert('RGB'), (0, index * height))
    return strip.quantize(colors=colors, method=Image.Quantize.MEDIANCUT, dither=dither)


def quantize_frames(frames: List[Image.Image], palette: Image.Image, colors: int, dither: int) -> List[Image.Image]:
    out: List[Image.Image] = []
    for frame in frames:
        quantized = frame.convert('RGB').quantize(colors=colors, palette=palette, dither=dither)
        out.append(quantized)
    return out


def main() -> int:
    args = parse_args()
    dither = Image.Dither.FLOYDSTEINBERG if args.dither else Image.Dither.NONE
    raw_frames = load_frames(args.inputs)
    processed = [
        preprocess(frame, contrast=args.contrast, cutoff=args.autocontrast_cutoff, sharpness=args.sharpness)
        for frame in raw_frames
    ]
    palette = build_global_palette(processed, colors=args.colors, dither=dither)
    frames = quantize_frames(processed, palette=palette, colors=args.colors, dither=dither)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    duration_ms = max(1, round(1000 / args.fps))
    first, *rest = frames
    first.save(
        output,
        save_all=True,
        append_images=rest,
        duration=duration_ms,
        loop=args.loop,
        optimize=False,
        disposal=2,
    )
    print(f'generated {output} ({len(frames)} frames, {duration_ms}ms/frame, {args.colors} colours)')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
