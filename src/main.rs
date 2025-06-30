use rltk::{GameState, RGB, Rltk};
use specs::prelude::*;

mod components;
pub use components::*;

mod map;
pub use map::*;

mod player;
use player::*;

mod rect;
pub use rect::Rect;

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        // Run systems here...
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // Update
        self.run_systems();
        player_input(self, ctx);

        // Draw
        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() -> rltk::BError {
    // Set up initial game context
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50().with_title("Roguelike").build()?;

    // Create game state and register ECS components
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.insert(new_map_test());

    // Create a player entity
    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    // Run the game's main loop
    rltk::main_loop(context, gs)
}
