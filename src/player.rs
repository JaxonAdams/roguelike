use crate::{Item, WantsToMelee, WantsToPickUpItem, gamelog::GameLog};

use super::{CombatStats, Map, Player, Position, RunState, State, Viewshed};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            if let Some(_t) = combat_stats.get(*potential_target) {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Failed to add melee target");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => {
                try_move_player(1, -1, &mut gs.ecs);
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => {
                try_move_player(-1, -1, &mut gs.ecs);
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                try_move_player(1, 1, &mut gs.ecs);
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                try_move_player(-1, 1, &mut gs.ecs);
            }

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push(String::from("There is nothing here to pick up.")),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickUpItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickUpItem {
                        collected_by: *player_entity,
                        item: item,
                    },
                )
                .expect("Unable to insert want to pickup.");
        }
    }
}
