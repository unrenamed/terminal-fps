use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use eyre::Result;
use std::io::{self, Write};
use std::time::{Duration, Instant};

mod player;
mod utils;

use player::Player;
use utils::*;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    execute!(lock, EnterAlternateScreen, cursor::Hide)?;

    let mut player = Player::new(2.0, 2.0, 0.0);

    let mut screen = [Shade::EMPTY; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut tp1 = Instant::now();

    loop {
        // Time differential per frame to ensure consistent movement when CPU loads/unloads
        let tp2 = Instant::now();
        let elapsed_time_duration = tp2 - tp1;
        tp1 = tp2;
        let elapsed_time = elapsed_time_duration.as_secs_f32();

        // Wait 5 millis for a keyboard event to occur
        if poll(Duration::from_millis(5))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Left | KeyCode::Char('a') => {
                        player.turn_left(SPEED * 0.75 * elapsed_time);
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        player.turn_right(SPEED * 0.75 * elapsed_time);
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        player.move_forward(
                            player.a().sin() * SPEED * elapsed_time,
                            player.a().cos() * SPEED * elapsed_time,
                        );
                        // Handle collision
                        if MAP[player.x() as usize * MAP_WIDTH + player.y() as usize] == '#' {
                            player.move_back(
                                player.a().sin() * SPEED * elapsed_time,
                                player.a().cos() * SPEED * elapsed_time,
                            );
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        player.move_back(
                            player.a().sin() * SPEED * elapsed_time,
                            player.a().cos() * SPEED * elapsed_time,
                        );
                        // Handle collision
                        if MAP[player.x() as usize * MAP_WIDTH + player.y() as usize] == '#' {
                            player.move_forward(
                                player.a().sin() * SPEED * elapsed_time,
                                player.a().cos() * SPEED * elapsed_time,
                            );
                        }
                    }
                    // Exit game
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        for x in 0..SCREEN_WIDTH {
            let ray_angle = (player.a() - FOV / 2.0) + (x as f32 / SCREEN_WIDTH as f32) * FOV;

            let mut distance_to_wall = 0.0 as f32;
            let step_size = 0.1 as f32;

            let mut hit_wall = false; // Set when a ray hits a wall
            let mut boundary = false; // Set when a ray hits a corner of a wall

            let eye_x = ray_angle.sin();
            let eye_y = ray_angle.cos();

            // Incrementally cast ray from player, along ray angle, testing for
            // intersection with a block
            while !hit_wall && distance_to_wall < DEPTH {
                distance_to_wall += step_size;
                let cx = (player.x() + eye_x * distance_to_wall) as usize;
                let cy = (player.y() + eye_y * distance_to_wall) as usize;
                if cx >= MAP_WIDTH || cy >= MAP_HEIGHT {
                    hit_wall = true;
                    distance_to_wall = DEPTH;
                } else if MAP[(cx * MAP_WIDTH + cy)] == '#' {
                    // Ray is inbounds so test to see if the ray cell is a wall block
                    hit_wall = true;

                    // To highlight wall boundaries, cast a ray from each corner
                    // of the tile to the player. The more coincident this ray
                    // is to the rendering ray, the closer we are to a tile
                    // boundary, which we'll shade to add detail to the walls
                    let mut wall_corners: Vec<(f32, f32)> = Vec::new();
                    for tx in 0..2 {
                        for ty in 0..2 {
                            let vy = (cy as f32) + (ty as f32) - player.y();
                            let vx = (cx as f32) + (tx as f32) - player.x();
                            let d = (vx * vx + vy * vy).sqrt();
                            let dot = (eye_x * vx / d) + (eye_y * vy / d);
                            wall_corners.push((d, dot));
                        }
                    }

                    // Sort Pairs from closest to farthest
                    wall_corners.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                    // First three are closest (we will never see all four)
                    let bound = 0.01;
                    if let [(_, y1), (_, y2), (_, y3), _] = wall_corners.as_slice() {
                        if y1.acos() < bound || y2.acos() < bound || y3.acos() < bound {
                            boundary = true;
                        }
                    }
                }
            }

            let ceiling = SCREEN_HEIGHT as f32 / 2.0 - SCREEN_HEIGHT as f32 / distance_to_wall;
            let floor = SCREEN_HEIGHT as f32 - ceiling;

            let mut shade: char;

            // Shade walls based on distance
            match distance_to_wall {
                d if d <= DEPTH / 4.0 => shade = Shade::WALL_FULL,
                d if d < DEPTH / 3.0 => shade = Shade::WALL_DARK,
                d if d < DEPTH / 2.0 => shade = Shade::WALL_MEDIUM,
                d if d < DEPTH => shade = Shade::WALL_LIGHT,
                _ => shade = Shade::EMPTY,
            }

            if boundary {
                shade = Shade::EMPTY;
            }

            for y in 0..SCREEN_HEIGHT {
                let idx = y * SCREEN_WIDTH + x;

                if y as f32 <= ceiling {
                    screen[idx] = Shade::EMPTY;
                } else if y as f32 > ceiling && y as f32 <= floor {
                    screen[idx] = shade;
                } else if y as f32 > floor {
                    // Shade floor based on distance
                    let b = 1.0
                        - ((y as f32 - SCREEN_HEIGHT as f32 / 2.0) / (SCREEN_HEIGHT as f32 / 2.0));
                    match b {
                        v if v < 0.25 => shade = Shade::FLOOR_DARK,
                        v if v < 0.5 => shade = Shade::FLOOR_MEDIUM,
                        v if v < 0.75 => shade = Shade::FLOOR_LIGHT,
                        v if v < 0.9 => shade = Shade::FLOOR_DIM,
                        _ => shade = Shade::EMPTY,
                    }
                    screen[idx] = shade;
                }
            }
        }

        // Build map
        for nx in 0..MAP_WIDTH {
            for ny in 0..MAP_HEIGHT {
                screen[ny * SCREEN_WIDTH + nx] = MAP[ny * MAP_WIDTH + nx];
            }
        }
        screen[player.x() as usize * SCREEN_WIDTH + player.y() as usize] = 'P';

        // Clean screen
        write!(lock, "\r\x1b[H")?;

        // Display stats
        writeln!(
            lock,
            "\r\x1b[KX: {:.2}, Y: {:.2}, A: {:.2}, FPS: {:.2}",
            player.x(),
            player.y(),
            player.a(),
            1.0 / elapsed_time
        )?;

        // Display frame with map
        write!(lock, "\r\x1b[K")?;
        for ny in 0..SCREEN_HEIGHT {
            for nx in 0..SCREEN_WIDTH {
                write!(lock, "{}", screen[ny * SCREEN_WIDTH + nx])?;
            }
            writeln!(lock, "\r")?; // Move to the beginning of the next line
        }

        lock.flush()?;
    }

    execute!(lock, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
