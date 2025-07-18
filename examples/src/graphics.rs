/// this example is for the new rendering API.

use ara::{
    render::{ RenderTo, ViewSystemExt },
    scene::{ Container, Graphics, ParentNode },
    Color,
    Context,
    Half,
};

pub fn run(gpu: Context) {
    let mut renderer = ara::render::Renderer::new(&gpu, ara::render::RendererSpecification {
        render_to: RenderTo {
            target: ara::render::ViewTarget::Empty, // todo change to image after we support offscreen rendering
            config: ara::render::ViewConfig {
                size: (800, 600).into(),
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
}
