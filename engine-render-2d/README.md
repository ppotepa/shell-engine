# engine-render-2d

2D rendering primitives shared by the compositor.

## Purpose

`engine-render-2d` owns reusable 2D sprite and layout rendering logic that does
not belong in compositor assembly:

- image rendering
- text rendering
- vector rendering
- panel/container helpers
- layout measurement and placement helpers

## Main exports

- `Render2dPipeline`
- `Render2dInput`
- image/text/vector render helpers
- layout measurement helpers and `RenderArea`

## Ownership split

- `engine-render-2d` owns 2D draw logic
- `engine-render-3d` owns 3D draw logic
- `engine-compositor` assembles final frames using both
