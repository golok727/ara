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

    // Aligns wont for work now :)
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
