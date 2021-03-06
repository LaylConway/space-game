use {
    ggez::{
        Context, GameResult,
        graphics::{self, MeshBuilder, DrawParam, Rect, spritebatch::{SpriteBatch}},
    },
    nalgebra::{Point2},

    spacegame_game::{
        object_class::{ObjectClasses},
        state::{GameState, Camera, ship::{Ship}},
    },
};

pub fn draw_ship(
    ctx: &mut Context,
    object_classes: &ObjectClasses,
    game_state: &GameState,
    tiles_batch: &mut SpriteBatch,
) -> GameResult<()> {

    draw_tiles(ctx, &game_state.ship, &game_state.camera, object_classes, tiles_batch)?;
    draw_tasks(ctx, &game_state.ship)?;
    draw_units(ctx, &game_state.ship)?;

    Ok(())
}

fn draw_tiles(
    ctx: &mut Context, ship: &Ship, camera: &Camera, object_classes: &ObjectClasses,
    tiles_batch: &mut SpriteBatch,
) -> GameResult<()> {
    let world_bounds = camera.world_bounds();

    for position in ship.tiles.bounds(world_bounds.0, world_bounds.1).iter() {
        let tile = ship.tiles.get(position).unwrap();

        let (fx, fy) = (position.x as f32, position.y as f32);

        // Add graphic for the floor
        if tile.floor {
            tiles_batch.add(DrawParam {
                src: Rect::new(0.0, 0.5, 0.5, 0.5),
                dest: Point2::new(fx, fy + 1.0),
                scale: Point2::new(1.0 / 64.0, -1.0 / 64.0),
                .. Default::default()
            });
        }

        // Add graphic for objects
        if let Some(ref object) = tile.object {
            let uvs = object_classes.get(object.class).unwrap().uvs;

            tiles_batch.add(DrawParam {
                src: uvs,
                dest: Point2::new(fx, fy + 1.0),
                scale: Point2::new(1.0 / 64.0, -1.0 / 64.0),
                .. Default::default()
            });
        }
    }

    graphics::set_color(ctx, (255, 255, 255).into())?;
    graphics::draw(ctx, tiles_batch, Point2::new(0.0, 0.0), 0.0)?;
    tiles_batch.clear();

    Ok(())
}

fn draw_tasks(
    ctx: &mut Context, ship: &Ship
) -> GameResult<()> {
    let mut tasks_builder = MeshBuilder::new();
    let mut unreachable_tasks_builder = MeshBuilder::new();

    for (_, task) in ship.task_queue.tasks() {
        let (fx, fy) = (task.position.x as f32, task.position.y as f32);

        let builder = if !task.unreachable {
            &mut tasks_builder
        } else {
            &mut unreachable_tasks_builder
        };

        // Add graphic for the task
        builder.triangles(&[
            Point2::new(fx + 0.25, fy + 0.25),
            Point2::new(fx + 0.75, fy + 0.25),
            Point2::new(fx + 0.25, fy + 0.75),

            Point2::new(fx + 0.75, fy + 0.75),
            Point2::new(fx + 0.25, fy + 0.75),
            Point2::new(fx + 0.75, fy + 0.25),
        ]);
    }

    let tasks_mesh = tasks_builder.build(ctx)?;
    let unreachable_tasks_mesh = unreachable_tasks_builder.build(ctx)?;

    graphics::set_color(ctx, (255, 255, 255, 25).into())?;
    graphics::draw(ctx, &tasks_mesh, Point2::new(0.0, 0.0), 0.0)?;

    graphics::set_color(ctx, (255, 120, 120, 50).into())?;
    graphics::draw(ctx, &unreachable_tasks_mesh, Point2::new(0.0, 0.0), 0.0)?;

    Ok(())
}

fn draw_units(
    ctx: &mut Context, ship: &Ship
) -> GameResult<()> {
    let mut units_builder = MeshBuilder::new();
    for unit in ship.units() {
        let pos = unit.position();
        units_builder.triangles(&[
            Point2::new(pos.x - 0.4, pos.y - 0.4),
            Point2::new(pos.x + 0.4, pos.y - 0.4),
            Point2::new(pos.x - 0.4, pos.y + 0.4),

            Point2::new(pos.x + 0.4, pos.y + 0.4),
            Point2::new(pos.x - 0.4, pos.y + 0.4),
            Point2::new(pos.x + 0.4, pos.y - 0.4),
        ]);
    }
    let units_mesh = units_builder.build(ctx)?;

    graphics::set_color(ctx, (150, 200, 150).into())?;
    graphics::draw(ctx, &units_mesh, Point2::new(0.0, 0.0), 0.0)?;

    Ok(())
}
