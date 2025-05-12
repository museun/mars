#[doc(inline)]
pub use mars_math::*;
#[doc(inline)]
pub use mars_surface::*;
#[doc(inline)]
pub use mars_terminal::*;

use std::time::{Duration, Instant};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Action {
    #[default]
    Continue,
    Quit,
}

pub trait Application {
    fn start(&mut self, size: Size, renderer: &mut impl RendererSetup) {
        _ = size;
        _ = renderer
    }

    fn stop(&mut self) {}

    fn should_quit(&self) -> bool {
        false
    }

    fn update(&mut self, update: Update) -> ShouldRender {
        _ = update;
        ShouldRender::Yes
    }

    fn event(&mut self, event: Event) -> Action {
        _ = event;
        Action::Continue
    }

    fn render(&mut self, renderer: &mut impl Renderer);
}

pub trait Runner: Application + Sized {
    fn run(self) -> std::io::Result<()> {
        let term = Terminal::create(Config::new())?;
        self::run(30.0, term, self)
    }

    fn debug(mut self) -> String {
        let size = Size::new(80, 24);

        let mut surface = BasicRenderer::new(size);
        let mut dr = DebugRasterizer::new();

        self.start(size, &mut surface);
        self.render(&mut surface);
        let Ok(..) = surface.render(&mut dr);
        self.stop();

        dr.to_string()
    }
}

impl<T> Runner for T where T: Application {}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum ShouldRender {
    #[default]
    Yes,
    No,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Update {
    pub last_frame: Instant,
    pub current: Instant,
    pub dt: f32,
    pub absolute_dt: f32,
}

pub fn run(fps: f32, mut term: Terminal, mut app: impl Application) -> std::io::Result<()> {
    assert!(fps >= 1.0, "fps must be atleast 1.0");

    let mut surface = BasicRenderer::new(term.size());
    let mut br = BufferedRasterizer::new();

    app.start(term.size(), &mut surface);

    // first render to clear the bg
    let (fg, bg) = surface.default_colors();
    surface.fill(
        Position::ZERO,
        term.size(),
        Pixel::empty().fg(fg).bg(bg),
        BlendMode::Replace,
    );
    let Ok(..) = surface.render(&mut br);
    let Ok(..) = br.clear_screen(bg, term.size());
    if let err @ Err(..) = br.copy_to(&mut term) {
        app.stop();
        return err;
    }

    let mut last_frame = Instant::now();
    let mut now = Instant::now();
    let mut lag = Duration::ZERO;
    let mut absolute_dt = 1.0;

    let target = 1.0 / fps;

    while !app.should_quit() {
        let update = Update {
            last_frame,
            current: now,
            dt: (now - last_frame).as_secs_f32(),
            absolute_dt,
        };

        let mut should_redraw = false;
        while let Some(ev) = term.try_read_event() {
            if ev.is_quit() {
                app.stop();
                return Ok(());
            }

            // TODO debounce these resizes
            if let Event::Resize { size } = &ev {
                surface.resize(*size, ResizeMode::Discard);
                should_redraw ^= true;
            }

            if let Action::Quit = app.event(ev) {
                app.stop();
                return Ok(());
            }
        }

        should_redraw ^= app.update(update) == ShouldRender::Yes;

        if should_redraw {
            // let (_, bg) = surface.default_colors();
            // let Ok(..) = br.clear_screen(bg, surface.size());

            app.render(&mut surface);
            let Ok(..) = surface.render(&mut br);

            if let err @ Err(_) = br.copy_to(&mut term) {
                app.stop();
                return err;
            }
        }

        let current = Instant::now();
        absolute_dt = current.duration_since(now).as_secs_f32();

        let remaining = Duration::from_secs_f32(target)
            .saturating_sub(current.duration_since(now))
            .saturating_sub(lag);
        std::thread::park_timeout(remaining);

        let next = Instant::now();
        lag = next.duration_since(current).saturating_sub(remaining);
        last_frame = std::mem::replace(&mut now, next)
    }

    app.stop();
    Ok(())
}
