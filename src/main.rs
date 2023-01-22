use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use eyre::Result;
use std::f32::consts::PI;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const SCREEN_WIDTH: usize = 120;
const SCREEN_HEIGHT: usize = 40;

const MAP_HEIGHT: usize = 16;
const MAP_WIDTH: usize = 16;

const FOV: f32 = PI / 4.0;
const DEPTH: f32 = 16.0;
const SPEED: f32 = 5.0;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    execute!(lock, EnterAlternateScreen, cursor::Hide)?;

    let mut player_x: f32 = 2.0;
    let mut player_y: f32 = 2.0;
    let mut player_a: f32 = 0.0;

    let mut screen = [' '; SCREEN_WIDTH * SCREEN_HEIGHT];
    let map = [
        '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '#', '#', '#', '#', '#', '#', '#', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '#', '#', '.', '.', '.', '#', '#', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
    ];

    let mut tp1 = Instant::now();

    loop {
        let tp2 = Instant::now();
        let elapsed_time_duration = tp2 - tp1;
        tp1 = tp2;
        let elapsed_time = elapsed_time_duration.as_secs_f32();

        if poll(Duration::from_millis(5))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Left | KeyCode::Char('a') => {
                        player_a -= SPEED * 0.75 * elapsed_time;
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        player_a += SPEED * 0.75 * elapsed_time;
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        player_x += player_a.sin() * SPEED * elapsed_time;
                        player_y += player_a.cos() * SPEED * elapsed_time;
                        if map[player_x as usize * MAP_WIDTH + player_y as usize] == '#' {
                            player_x -= player_a.sin() * SPEED * elapsed_time;
                            player_y -= player_a.cos() * SPEED * elapsed_time;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        player_x -= player_a.sin() * SPEED * elapsed_time;
                        player_y -= player_a.cos() * SPEED * elapsed_time;
                        if map[player_x as usize * MAP_WIDTH + player_y as usize] == '#' {
                            player_x += player_a.sin() * SPEED * elapsed_time;
                            player_y += player_a.cos() * SPEED * elapsed_time;
                        }
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        for x in 0..SCREEN_WIDTH {
            let ray_angle = (player_a - FOV / 2.0) + (x as f32 / SCREEN_WIDTH as f32) * FOV;

            let mut distance_to_wall = 0.0 as f32;
            let step_size = 0.1 as f32;

            let mut hit_wall = false;
            let mut boundary = false;

            let eye_x = ray_angle.sin();
            let eye_y = ray_angle.cos();

            while !hit_wall && distance_to_wall < DEPTH {
                distance_to_wall += step_size;
                let cx = (player_x + eye_x * distance_to_wall) as usize;
                let cy = (player_y + eye_y * distance_to_wall) as usize;
                if cx >= MAP_WIDTH || cy >= MAP_HEIGHT {
                    hit_wall = true;
                    distance_to_wall = DEPTH;
                } else if map[(cx * MAP_WIDTH + cy)] == '#' {
                    hit_wall = true;

                    let mut wall_corners: Vec<(f32, f32)> = Vec::new();
                    for tx in 0..2 {
                        for ty in 0..2 {
                            let vy = (cy as f32) + (ty as f32) - player_y;
                            let vx = (cx as f32) + (tx as f32) - player_x;
                            let d = (vx * vx + vy * vy).sqrt();
                            let dot = (eye_x * vx / d) + (eye_y * vy / d);
                            wall_corners.push((d, dot));
                        }
                    }

                    // sort Pairs from closest to farthest
                    wall_corners.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                    // first three are closest (we will never see all four)
                    let bound = 0.01;
                    if let Some(b) = wall_corners.get(0) {
                        if b.1.acos() < bound {
                            boundary = true;
                        }
                    }
                    if let Some(b) = wall_corners.get(1) {
                        if b.1.acos() < bound {
                            boundary = true;
                        }
                    }
                    if let Some(b) = wall_corners.get(2) {
                        if b.1.acos() < bound {
                            boundary = true;
                        }
                    }
                }
            }

            let ceiling = SCREEN_HEIGHT as f32 / 2.0 - SCREEN_HEIGHT as f32 / distance_to_wall;
            let floor = SCREEN_HEIGHT as f32 - ceiling;

            let mut shade: char;

            match distance_to_wall {
                d if d <= DEPTH / 4.0 => shade = '\u{2588}',
                d if d < DEPTH / 3.0 => shade = '\u{2593}',
                d if d < DEPTH / 2.0 => shade = '\u{2592}',
                d if d < DEPTH => shade = '\u{2591}',
                _ => shade = ' ',
            }

            if boundary {
                shade = ' ';
            }

            for y in 0..SCREEN_HEIGHT {
                let idx = y * SCREEN_WIDTH + x;

                if y as f32 <= ceiling {
                    screen[idx] = ' ';
                } else if y as f32 > ceiling && y as f32 <= floor {
                    screen[idx] = shade;
                } else {
                    let b = 1.0
                        - ((y as f32 - SCREEN_HEIGHT as f32 / 2.0) / (SCREEN_HEIGHT as f32 / 2.0));
                    match b {
                        v if v < 0.25 => shade = '#',
                        v if v < 0.5 => shade = 'x',
                        v if v < 0.75 => shade = '~',
                        v if v < 0.9 => shade = '-',
                        _ => shade = ' ',
                    }
                    screen[idx] = shade;
                }
            }
        }

        // build map
        for nx in 0..MAP_WIDTH {
            for ny in 0..MAP_HEIGHT {
                screen[(ny) * SCREEN_WIDTH + nx] = map[ny * MAP_WIDTH + nx];
            }
        }
        screen[player_x as usize * SCREEN_WIDTH + player_y as usize] = 'P';

        // clean screen
        write!(lock, "\r\x1b[H")?;
        
        // display stats
        writeln!(
            lock,
            "\r\x1b[KX: {:.2}, Y: {:.2}, A: {:.2}, FPS: {:.2}",
            player_x,
            player_y,
            player_a,
            1.0 / elapsed_time
        )?;

        // display frame with map
        write!(lock, "\r\x1b[K")?;
        for ny in 0..SCREEN_HEIGHT {
            for nx in 0..SCREEN_WIDTH {
                write!(lock, "{}", screen[ny * SCREEN_WIDTH + nx])?;
            }
            writeln!(lock, "\r")?; // move to the beginning of the next line
        }

        lock.flush()?;
    }

    execute!(lock, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
