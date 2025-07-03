use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

mod gamelog;
mod gui;
mod spawner;

mod components;
pub use components::*;

mod map;
pub use map::*;

mod player;
use player::*;

mod rect;
pub use rect::Rect;

mod visibility_system;
pub use visibility_system::VisibilitySystem;

mod monster_ai_system;
pub use monster_ai_system::MonsterAI;

mod map_indexing_system;
pub use map_indexing_system::MapIndexingSystem;

mod melee_combat_system;
pub use melee_combat_system::MeleeCombatSystem;

mod damage_system;
pub use damage_system::DamageSystem;

mod inventory_system;
pub use inventory_system::ItemCollectionSystem;

use crate::gui::draw_ui;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
}

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);

        let mut melee_combat = MeleeCombatSystem {};
        melee_combat.run_now(&self.ecs);

        let mut dmg_sys = DamageSystem {};
        dmg_sys.run_now(&self.ecs);

        let mut pickup_system = ItemCollectionSystem {};
        pickup_system.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        // Run game systems and update state
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                // Render will happen *after* entity drawing
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        // Render game map and entities
        draw_map(&self.ecs, ctx);

        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        draw_ui(&self.ecs, ctx);

        match newrunstate {
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                if result == gui::ItemMenuResult::Cancel {
                    let mut runwriter = self.ecs.write_resource::<RunState>();
                    *runwriter = RunState::AwaitingInput;
                }
            }
            _ => {}
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    // Set up initial game context
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50().with_title("Roguelike").build()?;
    context.with_post_scanlines(true);

    // Create game state and register ECS components
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<HealingPotion>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickUpItem>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Jaxon's Roguelike!".to_string()],
    });
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);

    // Run the game's main loop
    rltk::main_loop(context, gs)
}
