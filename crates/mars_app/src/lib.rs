use std::time::{Duration, Instant};

use mars_math::Size;
use mars_surface::{BufferedRasterizer, ResizeMode, SurfaceRenderer};
use mars_terminal::{Action, Context, Event, Terminal};

pub trait Application {
    fn start(&mut self, size: Size, ctx: Context) {
        _ = size;
        _ = ctx;
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

    fn render(&mut self, surface: &mut SurfaceRenderer);
}

pub trait Runner {
    fn run(self) -> std::io::Result<()>;
}

impl<T> Runner for T
where
    T: Application,
{
    fn run(self) -> std::io::Result<()> {
        let term = Terminal::create()?;
        self::run(30.0, term, self)
    }
}

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

    let context = Context::new(&term);
    let mut surface = SurfaceRenderer::new(term.size());
    let mut br = BufferedRasterizer::new();

    app.start(term.size(), context);

    let mut last_frame = Instant::now();
    let mut now = Instant::now();
    let mut lag = Duration::ZERO;
    let mut absolute_dt = 1.0;

    let target = 1.0 / fps;

    while !app.should_quit() {
        while let Some(ev) = term.try_read_event() {
            if ev.is_quit() {
                break;
            }

            // TODO debounce these resizes
            if let Event::Resize { size } = &ev {
                surface.resize(*size, ResizeMode::Discard)
            }

            if let Action::Quit = app.event(ev) {
                break;
            }
        }

        let update = Update {
            last_frame,
            current: now,
            dt: (now - last_frame).as_secs_f32(),
            absolute_dt,
        };

        if let ShouldRender::Yes = app.update(update) {
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
