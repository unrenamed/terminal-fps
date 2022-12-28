use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use eyre::Result;
use std::f32::consts::PI;
use std::io::{self, Write};
use std::time::{Instant, Duration};

const SCREEN_WIDTH: usize = 230;
const SCREEN_HEIGHT: usize = 50;

const MAP_HEIGHT: usize = 16;
const MAP_WIDTH: usize = 16;

const FOV: f32 = PI / 3.0;
const DEPTH: f32 = 30.0;
const SPEED: f32 = 5.0;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    execute!(lock, EnterAlternateScreen, cursor::Hide)?;

    let mut player_x: f32 = 2.0;
    let mut player_y: f32 = 1.0;
    let mut player_a: f32 = 0.0;

    let mut screen = [' '; SCREEN_WIDTH * SCREEN_HEIGHT];
    let map = [
        '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '.', '.', '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '#',
        '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
    ];

    let mut elapsed_time = Duration::from_millis(0);

    loop {
        let now = Instant::now();
        
        for x in 0..SCREEN_WIDTH {
            let ray_angle = (player_a - FOV / 2.0) + (x as f32 / SCREEN_WIDTH as f32) * FOV;

            let mut distance_to_wall = 0.0 as f32;
            let step_size = 0.1 as f32;
            let mut hit_wall = false;

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
                }
            }

            let ceiling = SCREEN_HEIGHT as f32 / 2.0 - SCREEN_HEIGHT as f32 / distance_to_wall;
            let floor = SCREEN_HEIGHT as f32 - ceiling;

            let mut shade: char;

            match distance_to_wall {
                d if d <= DEPTH / 3.0 => shade = '\u{2588}',
                d if d < DEPTH / 2.0 => shade = '\u{2593}',
                d if d < DEPTH / 1.5 => shade = '\u{2592}',
                d if d < DEPTH => shade = '\u{2591}',
                _ => shade = ' ',
            }

            for y in 0..SCREEN_HEIGHT {
                let idx = y * SCREEN_WIDTH + x;

                if y as f32 <= ceiling {
                    screen[idx] = ' ';
                } else if y as f32 > ceiling && y as f32 <= floor {
                    screen[idx] = shade;
                } else {
                    let b = 1.0 - ((y as f32 - SCREEN_HEIGHT as f32 / 2.0) / (SCREEN_HEIGHT as f32 / 2.0));
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
                screen[(ny + 1) * SCREEN_WIDTH + nx] = map[ny * MAP_WIDTH + nx];
            }
        }
        screen[(player_x as usize + 1) * SCREEN_WIDTH + player_y as usize] = 'P';

        // display stats
        writeln!(lock, "\r\x1b[H")?;
        writeln!(lock, "\r\x1b[KX: {}, Y: {}, A: {}, FPS: {}", player_x, player_y, player_a, 60)?;
        
         // display frame with map
         write!(lock, "\r\x1b[K")?;
         for ny in 0..SCREEN_HEIGHT {
             for nx in 0..SCREEN_WIDTH {
                 write!(lock, "{}", screen[ny * SCREEN_WIDTH + nx])?;
             }
             writeln!(lock, "\r")?; // move to the beginning of the next line
         }

        lock.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Left | KeyCode::Char('a') => {
                    player_a -= SPEED * 0.05;
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    player_a += SPEED * 0.05;
                }
                KeyCode::Up | KeyCode::Char('w') => {
                    player_x += player_a.sin() * SPEED * 0.15;
                    player_y += player_a.cos() * SPEED * 0.15;
                    if map[player_x as usize * MAP_WIDTH + player_y as usize] == '#' {
                        player_x -= player_a.sin() * SPEED * 0.15;
                        player_y -= player_a.cos() * SPEED * 0.15;
                    }
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    player_x -= player_a.sin() * SPEED * 0.15;
                    player_y -= player_a.cos() * SPEED * 0.15;
                    if map[player_x as usize * MAP_WIDTH + player_y as usize] == '#' {
                        player_x += player_a.sin() * SPEED * 0.15;
                        player_y += player_a.cos() * SPEED * 0.15;
                    }
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }

        elapsed_time = now.elapsed();
    }

    execute!(lock, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}