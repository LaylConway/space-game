extern crate ggez;
extern crate alga;
extern crate nalgebra;
#[macro_use]
extern crate slog;
extern crate sloggers;
extern crate metrohash;
extern crate pathfinding;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate markedly;
extern crate markedly_ggez;

mod controller;
pub mod model;
mod view;

use std::env;
use std::path;

use ggez::{Context, GameResult, GameError};
use ggez::timer;
use ggez::conf::{Conf, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, MouseButton, MouseState};
use ggez::graphics::spritebatch::{SpriteBatch};
use ggez::graphics::{self, Font, Text, Image, Rect};
use nalgebra::{Vector2, Point2};
use slog::{Logger};
use sloggers::{Build};
use sloggers::terminal::{TerminalLoggerBuilder};
use sloggers::types::{Severity};

use markedly::class::{ComponentClasses};
use markedly::input::{UiInput};
use markedly::scripting::{ScriptRuntime};
use markedly::template::{Template, Style};
use markedly::{Ui, ComponentEvents, UiContext};
use markedly_ggez::{GgezRenderer, GgezComponentCache};

use controller::{BuildInputController, CameraInputController, SaveInputController};
use model::{Camera, ObjectClasses, GenericObjectClass};
use model::ship::{Ship};

pub fn main() {
    // Set up logging
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    let log = builder.build().unwrap();

    // Set up the ggez context
    let mut c = Conf::new();
    c.window_mode = WindowMode {
        width: 1280,
        height: 720,
        .. Default::default()
    };
    c.window_setup = WindowSetup {
        title: "Space Game".into(),
        .. Default::default()
    };
    let ctx = &mut Context::load_from_conf("space-game", "carbidegames", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so we we look in the cargo
    // project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    // Initialize and run the game
    let result = MainState::new(ctx, log.clone())
        .and_then(|mut s| event::run(ctx, &mut s));

    // Check if it ran successfully
    if let Err(e) = result {
        match e {
            GameError::UnknownError(text) => error!(log, "Fatal:\n{}", text),
            e => error!(log, "Fatal: {}", e)
        }
    } else {
        info!(log, "Game exited cleanly");
    }
}

struct MainState {
    log: Logger,

    // Ui
    ui_context: UiContext,
    ui: Ui,
    ui_input: UiInput,
    ui_font: Font,
    ui_cache: GgezComponentCache,
    root_events: ComponentEvents,

    // Models
    camera: Camera,
    object_classes: ObjectClasses,
    ship: Ship,

    // Controllers
    build_input: BuildInputController,
    camera_input: CameraInputController,
    save_input: SaveInputController,

    // View Data
    tiles: SpriteBatch,
}

impl MainState {
    fn new(ctx: &mut Context, log: Logger) -> GameResult<MainState> {
        info!(log, "Loading game");

        // Set up the game world camera
        let screen_size = Vector2::new(1280, 720);
        let mut camera = Camera::new(64, screen_size);
        camera.set_position(Point2::new(50.0, 50.0));

        // Set up everything needed for the UI
        let mut classes = ComponentClasses::new();
        classes.register::<markedly::class::ContainerClass>("container");
        classes.register::<markedly::class::ButtonClass>("button");

        let ui_input = UiInput::new();
        let ui_font = Font::new(ctx, "/DejaVuSansMono.ttf", 8)?;
        let ui_context = UiContext {
            classes,
            runtime: ScriptRuntime::new(),
        };

        // Set up the UI itself
        let style = Style::from_reader(ctx.filesystem.open("/markedly/style.mark")?)?;
        let root_template = Template::from_reader(ctx.filesystem.open("/markedly/root.mark")?)?;
        let (mut ui, root_events) = Ui::new(
            &root_template, style,
            Vector2::new(screen_size.x as f32, screen_size.y as f32),
            &ui_context
        ).map_err(|e| format!("{:#?}", e))?;

        // Set up all the objects we can place in ships
        let mut object_classes = ObjectClasses::new();
        object_classes.register(GenericObjectClass {
            uvs: Rect::new(0.0, 0.0, 0.5, 0.5), walkable: false,
        });
        object_classes.register(GenericObjectClass {
            uvs: Rect::new(0.5, 0.0, 0.5, 0.5), walkable: true,
        });

        // Create the starter ship
        let ship = Ship::starter(&log);

        let build_input = BuildInputController::new(ctx, &mut ui, &ui_context)?;
        let camera_input = CameraInputController::new();
        let save_input = SaveInputController::new(ctx, &mut ui, &ui_context)?;

        let tiles = SpriteBatch::new(Image::new(ctx, "/tiles.png")?);

        Ok(MainState {
            log,

            ui_context,
            ui,
            ui_input,
            ui_font,
            ui_cache: GgezComponentCache::new(),
            root_events,

            camera,
            object_classes,
            ship,

            build_input,
            camera_input,
            save_input,

            tiles,
        })
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        const DELTA: f32 = 1.0 / DESIRED_FPS as f32;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            while let Some(_) = self.root_events.next() {}

            self.build_input.update()?;
            self.save_input.update(&self.log, ctx, &mut self.ship)?;
            self.ship.update(&self.log, DELTA, &self.object_classes);

            self.ui.update_ui_from_models(&self.ui_context)
                .map_err(|e| format!("{:#?}", e))?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, (10, 10, 15).into());
        graphics::clear(ctx);

        // Switch the projection to world rendering
        let size = graphics::get_size(ctx);
        self.camera.set_screen_size(Vector2::new(size.0 as i32, size.1 as i32));
        let pixels_projection = graphics::get_projection(ctx);
        graphics::set_projection(ctx, self.camera.projection());
        graphics::apply_transformations(ctx)?;

        // Draw everything in the world
        view::draw_ship(ctx, &self.ship, &self.camera, &self.object_classes, &mut self.tiles)?;
        view::draw_build_graphics(
            ctx, &self.build_input, &self.ship, &self.camera, &self.object_classes, &mut self.tiles
        )?;

        // Swith the projection back to pixels rendering for UI
        graphics::set_projection(ctx, pixels_projection);
        graphics::apply_transformations(ctx)?;

        // Draw the UI
        {
            let mut renderer = GgezRenderer::new(ctx, &mut self.ui_cache, &self.ui_font);
            markedly::render::render(&mut renderer, &mut self.ui)
                .map_err(|e| format!("{:#?}", e))?;
        }

        // Draw an FPS counter
        let fps = timer::get_fps(ctx);
        let text = Text::new(ctx, &format!("FPS: {:.2}", fps), &self.ui_font)?;
        graphics::set_color(ctx, (255, 255, 255, 200).into())?;
        graphics::draw(ctx, &text, Point2::new(0.0, 710.0), 0.0)?;

        graphics::present(ctx);
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut Context,
        button: MouseButton, x: i32, y: i32
    ) {
        self.ui_input.handle_drag_started(Point2::new(x as f32, y as f32), &mut self.ui);

        self.build_input.handle_mouse_down(button);
        self.camera_input.handle_mouse_down(button);
    }

    fn mouse_button_up_event(
        &mut self, _ctx: &mut Context,
        button: MouseButton, x: i32, y: i32
    ) {
        self.ui_input.handle_drag_ended(Point2::new(x as f32, y as f32), &mut self.ui);

        self.build_input.handle_mouse_up(button, &mut self.ship);
        self.camera_input.handle_mouse_up(button);
    }

    fn mouse_motion_event(
        &mut self, _ctx: &mut Context,
        _state: MouseState, x: i32, y: i32, xrel: i32, yrel: i32
    ) {
        let position = Point2::new(x, y);
        let rel_position = Vector2::new(xrel, yrel);

        self.ui_input.handle_cursor_moved(Point2::new(x as f32, y as f32), &mut self.ui);

        self.build_input.handle_mouse_move(position, &mut self.camera, &self.ship, &self.ui_input);
        self.camera_input.handle_mouse_move(rel_position, &mut self.camera);
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        info!(self.log, "quit_event() callback called, quitting");
        false
    }
}
