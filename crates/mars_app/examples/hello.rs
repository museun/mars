use mars_app::{Application, Runner, ShouldRender, Update};
use mars_math::{Anchor2, Size};
use mars_surface::{BlendMode, Drawable as _, DrawableExt as _, Renderer, RendererSetup, Rgba};

fn main() -> std::io::Result<()> {
    App::default().run()
}

#[derive(Default)]
struct App {
    dt: f32,
}

impl Application for App {
    fn start(&mut self, _size: Size, renderer: &mut impl RendererSetup) {
        renderer.set_default_bg(Rgba::hex("#538"));
    }

    fn update(&mut self, update: Update) -> ShouldRender {
        self.dt += update.dt;
        ShouldRender::Yes
    }

    fn render(&mut self, renderer: &mut impl Renderer) {
        // fn inverse_lerp(a: f32, b: f32, t: f32) -> f32 {
        //     if a == b {
        //         return 0.0;
        //     }
        //     (t - a) / (b - a)
        // }

        // let pulse = 0.5;
        // let w = 5;
        // let h = 3;

        // let size = renderer.size();
        // let (sin, cos) = self.dt.sin_cos();

        // renderer.fill_with(Position::new(10, 10), size / 2, |pos| {
        //     let color = if (((pos.x / w) % 2) ^ ((pos.y / h) % 2)) == 0 {
        //         sin
        //     } else {
        //         cos
        //     };
        //     let l = inverse_lerp(0.0, size.width as f32 - 1.0, pos.x as f32);
        //     let color = (pulse * color * 0.5 + 0.5).abs() + (1.0 * l);
        //     let rgba = [color + l, color - l, color * l, 1.0];
        //     Pixel::new(' ').bg(Rgba::from_float(rgba))
        // });

        // "hello world"
        //     .with_anchor(Anchor2::CENTER_CENTER)
        //     .with_bg(Rgba::hex("#000"))
        //     .with_fg(Rgba::hex("#0FF"))
        //     .render(renderer, BlendMode::Replace);

        for (anchor, name) in [
            (Anchor2::LEFT_TOP, "LEFT_TOP"),
            (Anchor2::RIGHT_TOP, "RIGHT_TOP"),
            (Anchor2::CENTER_TOP, "CENTER_TOP"),
            (Anchor2::LEFT_CENTER, "LEFT_CENTER"),
            (Anchor2::CENTER_CENTER, "CENTER_CENTER"),
            (Anchor2::RIGHT_CENTER, "RIGHT_CENTER"),
            (Anchor2::LEFT_BOTTOM, "LEFT_BOTTOM"),
            (Anchor2::CENTER_BOTTOM, "CENTER_BOTTOM"),
            (Anchor2::RIGHT_BOTTOM, "RIGHT_BOTTOM"),
        ] {
            format!("hello world\nthis is at {name}")
                .with_fg(Rgba::hex("#FFF"))
                .with_bg(Rgba::hex("#9988b3"))
                .with_anchor(anchor)
                .render(renderer, BlendMode::Replace);
        }
    }
}
