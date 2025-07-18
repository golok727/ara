# Ara

**Ara** is a 2D GPU-accelerated rendering library built on top of [`wgpu`](https://github.com/gfx-rs/wgpu).  
It is the second iteration of [Skie](https://github.com/golok727/saki/tree/main/skie), with a focus on retained-mode rendering and future extensibility.

Ara serves as the **primary rendering engine** for [@flutebrush](https://github.com/flutebrush).

---

## 🚧 Project Status

- This repository is **synced from the main Flutebrush repo**.
- **The API is actively evolving** and may change frequently — use at your own risk.
- **Immediate mode API will be deprecated** in favor of the retained mode system.

---

## ✨ Features

- Powered by `wgpu`, offering cross-platform hardware acceleration
- Supports **retained-mode** rendering via a scene graph
- Legacy **immediate-mode API** from Skie (soon to be removed)

---

## 🧪 Running Examples

To run a specific example:

```bash
cargo run -p ara-examples <example-name>
```
Replace <example-name> with one of the examples located in the examples/ directory.


## 🎨 Retained Mode API (Preferred)
```rs
use ara::{
    gpu::{Context, ContextSpecification, PowerPreference, Backends},
    render::{Renderer, RendererSpecification, RenderTo, ViewConfig},
    shape::{Container, Graphics},
    paint::Color,
};

let gpu = Context::new(ContextSpecification {
    power_preference: PowerPreference::HighPerformance,
    backends: Backends::all(),
    ..Default::default()
}).await.expect("Failed to create GPU context");

let window = /* your window, e.g. from winit */;

let mut renderer = Renderer::new(&gpu, RendererSpecification {
    render_to: RenderTo {
        target: window,
        config: ViewConfig {
            size: (800, 600).into(),
            resolution: dpi as f32,
            ..Default::default()
        },
    },
    ..Default::default()
});

renderer.init();

let screen_size = renderer.screen_size();
let mut stage = Container::default();
let mut g = Graphics::default();
let center = screen_size.map(|s| s as f32).half();

g.rect((10.0, 10.0, 200.0, 100.0))
 .fill(Color::RED)
 .circle((10.0, 10.0), 50.0)
 .fill(Color::KHAKI)
 .circle(center, 100.0)
 .fill(Color::KHAKI)
 .line_width(10)
 .stroke(Color::RED);

stage.child(&g);
renderer.render(&stage, ());
```

# ⚠️ Immediate Mode API (Legacy)
Immediate mode is inherited from Skie and will be removed in future releases.
```rs
use ara::{ gpu, vec2, Brush, Canvas, CanvasConfig, Color, Context, Corners, Half, Rect, Text };

pub fn run(gpu: Context) {
    let mut canvas = Canvas::new(
        gpu.clone(),
        CanvasConfig::default()
            .width(1024)
            .height(1024)
            .msaa_samples(ara::MsaaSampleLevel::Four)
            .add_surface_usage(gpu::TextureUsages::COPY_SRC)
    );

    let mut context = canvas.create_offscreen_context();
    let size = canvas.screen().map(|v| v as f32);

    let rect = Rect::xywh(size.width.half(), size.height.half(), 500.0, 500.0).centered();

    canvas.draw_round_rect(
        &rect,
        &Corners::with_all(10.0),
        Brush::filled(Color::TORCH_RED).stroke_color(Color::WHITE).line_width(5).antialias(true)
    );

    let center = rect.center();
    canvas.draw_circle(center.x, center.y, 200.0, Brush::filled(Color::WHITE).antialias(true));

    let pos = center - vec2(150.0, 50.0);
    let text = Text::new("✨ Ara ✨").pos(pos.x, pos.y).size_px(64.0);
    canvas.fill_text(&text, Color::BLACK);

    canvas.clear_color(Color::THAMAR_BLACK);
    canvas.render(&mut context).expect("error painting");

    let snapshot = canvas.snapshot_sync(&context).expect("Error taking snapshot");

    let image_buffer = image::ImageBuffer::<image::Rgba<u8>, _>
        ::from_raw(snapshot.size.width, snapshot.size.height, snapshot.data)
        .expect("Failed to create image buffer");

    let out_dir = std::path::Path::new("output");
    std::fs::create_dir_all(out_dir).expect("Error creating output dir");

    let out_path = out_dir.join("render.png");
    image_buffer.save(out_path.clone()).expect("Failed to save image");

    println!("Saved to {}", out_path.to_string_lossy());
}
```
