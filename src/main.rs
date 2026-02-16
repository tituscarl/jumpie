use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

// ── Constants ──────────────────────────────────────────────────────────
const PLAYER_COL: u16 = 4;
const INITIAL_SPEED_MS: u64 = 50;
const GRAVITY: f64 = 0.6;
const JUMP_VELOCITY: f64 = -3.8;
const MIN_OBSTACLE_GAP: u16 = 18;
const MAX_OBSTACLE_GAP: u16 = 35;

// ── Obstacle ───────────────────────────────────────────────────────────
#[derive(Clone)]
struct Obstacle {
    x: u16,
    height: u16, // 1, 2, or 3 blocks tall
}

// ── Draw helpers ───────────────────────────────────────────────────────
fn draw_ground(stdout: &mut impl Write, ground_y: u16, width: u16) -> std::io::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, ground_y),
        SetForegroundColor(Color::DarkGreen)
    )?;
    for _ in 0..width {
        write!(stdout, "▔")?;
    }
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn draw_player(
    stdout: &mut impl Write,
    col: u16,
    ground_y: u16,
    y_offset: f64,
) -> std::io::Result<()> {
    let top = (ground_y as f64 + y_offset - 2.0).max(0.0) as u16;
    execute!(
        stdout,
        SetForegroundColor(Color::Yellow),
        SetAttribute(Attribute::Bold)
    )?;
    // 3-row sprite
    execute!(stdout, cursor::MoveTo(col, top))?;
    write!(stdout, " O ")?;
    execute!(stdout, cursor::MoveTo(col, top + 1))?;
    write!(stdout, "/|\\")?;
    execute!(stdout, cursor::MoveTo(col, top + 2))?;
    write!(stdout, "/ \\")?;
    execute!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn draw_obstacle(stdout: &mut impl Write, obs: &Obstacle, ground_y: u16) -> std::io::Result<()> {
    execute!(stdout, SetForegroundColor(Color::Red))?;
    for h in 0..obs.height {
        let y = ground_y - 1 - h;
        execute!(stdout, cursor::MoveTo(obs.x, y))?;
        write!(stdout, "█")?;
    }
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn draw_score(
    stdout: &mut impl Write,
    score: u32,
    high_score: u32,
    width: u16,
) -> std::io::Result<()> {
    // Score left
    execute!(
        stdout,
        cursor::MoveTo(2, 1),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold)
    )?;
    write!(stdout, "Score: {}", score)?;
    // High score right
    let hs_text = format!("Best: {}", high_score);
    let hs_col = width.saturating_sub(hs_text.len() as u16 + 2);
    execute!(
        stdout,
        cursor::MoveTo(hs_col, 1),
        SetForegroundColor(Color::Magenta)
    )?;
    write!(stdout, "{}", hs_text)?;
    execute!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn draw_controls(stdout: &mut impl Write, width: u16) -> std::io::Result<()> {
    let msg = "SPACE / ↑ to jump  |  P pause  |  Q quit";
    let col = (width / 2).saturating_sub(msg.len() as u16 / 2);
    execute!(
        stdout,
        cursor::MoveTo(col, 3),
        SetForegroundColor(Color::DarkGrey)
    )?;
    write!(stdout, "{}", msg)?;
    execute!(stdout, ResetColor)?;
    Ok(())
}

fn center_text(
    stdout: &mut impl Write,
    text: &str,
    y: u16,
    width: u16,
    color: Color,
) -> std::io::Result<()> {
    let col = (width / 2).saturating_sub(text.len() as u16 / 2);
    execute!(
        stdout,
        cursor::MoveTo(col, y),
        SetForegroundColor(color),
        SetAttribute(Attribute::Bold)
    )?;
    write!(stdout, "{}", text)?;
    execute!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;
    Ok(())
}

// ── Screens ────────────────────────────────────────────────────────────
fn show_title_screen(
    stdout: &mut impl Write,
    width: u16,
    height: u16,
    high_score: u32,
) -> std::io::Result<()> {
    execute!(stdout, terminal::Clear(ClearType::All))?;
    let mid = height / 2;
    let logo = [
        r"     _                       _      ",
        r"    | |_   _ _ __ ___  _ __ (_) ___ ",
        r" _  | | | | | '_ ` _ \| '_ \| |/ _ \",
        r"| |_| | |_| | | | | | | |_) | |  __/",
        r" \___/ \__,_|_| |_| |_| .__/|_|\___|",
        r"                       |_|           ",
    ];
    for (i, line) in logo.iter().enumerate() {
        center_text(stdout, line, mid - 6 + i as u16, width, Color::Yellow)?;
    }
    center_text(
        stdout,
        "── A Terminal Jumping Game ──",
        mid - 0,
        width,
        Color::DarkGrey,
    )?;
    center_text(
        stdout,
        "Press SPACE or ENTER to start",
        mid + 2,
        width,
        Color::Green,
    )?;
    center_text(stdout, "Press Q to quit", mid + 4, width, Color::DarkGrey)?;
    if high_score > 0 {
        let hs = format!("High Score: {}", high_score);
        center_text(stdout, &hs, mid + 6, width, Color::Magenta)?;
    }
    stdout.flush()?;
    Ok(())
}

fn show_game_over(
    stdout: &mut impl Write,
    width: u16,
    height: u16,
    score: u32,
    high_score: u32,
    is_new_best: bool,
) -> std::io::Result<()> {
    let mid = height / 2;
    center_text(
        stdout,
        "╔══════════════════════════╗",
        mid - 3,
        width,
        Color::Red,
    )?;
    center_text(
        stdout,
        "║       GAME  OVER         ║",
        mid - 2,
        width,
        Color::Red,
    )?;
    center_text(
        stdout,
        "╚══════════════════════════╝",
        mid - 1,
        width,
        Color::Red,
    )?;
    let score_msg = format!("Score: {}", score);
    center_text(stdout, &score_msg, mid + 1, width, Color::Cyan)?;
    if is_new_best {
        center_text(stdout, "★ NEW HIGH SCORE! ★", mid + 3, width, Color::Yellow)?;
    } else {
        let best_msg = format!("Best: {}", high_score);
        center_text(stdout, &best_msg, mid + 3, width, Color::Magenta)?;
    }
    center_text(
        stdout,
        "Press SPACE/ENTER to play again",
        mid + 5,
        width,
        Color::Green,
    )?;
    center_text(stdout, "Press Q to quit", mid + 7, width, Color::DarkGrey)?;
    stdout.flush()?;
    Ok(())
}

fn show_pause(stdout: &mut impl Write, width: u16, height: u16) -> std::io::Result<()> {
    let mid = height / 2;
    center_text(stdout, "║  PAUSED  ║", mid - 1, width, Color::Yellow)?;
    center_text(stdout, "Press P to resume", mid + 1, width, Color::DarkGrey)?;
    stdout.flush()?;
    Ok(())
}

// ── Speed curve (fixed!) ───────────────────────────────────────────────
fn speed_for_score(score: u32) -> u64 {
    // Smooth speed-up: starts at INITIAL_SPEED_MS, asymptotically approaches 12 ms
    let base = INITIAL_SPEED_MS as f64;
    let min_speed = 12.0;
    let decay = (-0.04 * score as f64).exp(); // exponential decay
    (min_speed + (base - min_speed) * decay) as u64
}

// ── Collision ──────────────────────────────────────────────────────────
fn check_collision(obstacles: &[Obstacle], ground_y: u16, player_y: f64, player_col: u16) -> bool {
    let player_left = player_col;
    let player_right = player_col + 2; // sprite is 3 chars wide
    let player_top = (ground_y as f64 + player_y - 2.0) as u16;
    let player_bottom = ground_y as f64 + player_y;

    for obs in obstacles {
        // Obstacle occupies column obs.x, rows (ground_y-1) down to (ground_y - obs.height)
        if obs.x >= player_left && obs.x <= player_right {
            let obs_top = ground_y - obs.height;
            if (player_bottom as u16) >= obs_top {
                return true;
            }
            // Also check the player_top reaching into obstacle
            if player_top >= obs_top && player_top <= ground_y - 1 {
                return true;
            }
        }
    }
    false
}

// ── Main ───────────────────────────────────────────────────────────────
fn main() -> std::io::Result<()> {
    // Handle --version flag for Homebrew formula test
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("jumpie {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut high_score: u32 = 0;

    'outer: loop {
        let (screen_width, screen_height) = terminal::size()?;
        let ground_y = screen_height - 2;

        // ── Title screen ───────────────────────────────────────────
        show_title_screen(&mut stdout, screen_width, screen_height, high_score)?;
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break 'outer,
                        KeyCode::Enter | KeyCode::Char(' ') => break,
                        _ => {}
                    }
                }
            }
        }

        // ── Game state ─────────────────────────────────────────────
        let mut rng = rand::thread_rng();
        let mut player_y: f64 = 0.0; // offset from ground (0 = on ground, negative = in air)
        let mut velocity: f64 = 0.0;
        let mut jumping = false;
        let mut score: u32 = 0;
        let mut paused = false;
        let mut obstacles: Vec<Obstacle> = vec![Obstacle {
            x: screen_width,
            height: rng.gen_range(1..=2),
        }];
        let mut _frame: u64 = 0;

        let game_start = Instant::now();

        // ── Game loop ──────────────────────────────────────────────
        loop {
            let speed = speed_for_score(score);

            // Handle input
            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break 'outer,
                        KeyCode::Char('p') => {
                            paused = !paused;
                            if paused {
                                show_pause(&mut stdout, screen_width, screen_height)?;
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Up
                            if !jumping && !paused =>
                        {
                            jumping = true;
                            velocity = JUMP_VELOCITY;
                        }
                        _ => {}
                    }
                }
            }

            if paused {
                sleep(Duration::from_millis(50));
                continue;
            }

            // ── Physics ────────────────────────────────────────────
            if jumping {
                player_y += velocity;
                velocity += GRAVITY;
                if player_y >= 0.0 {
                    player_y = 0.0;
                    jumping = false;
                    velocity = 0.0;
                }
            }

            // ── Move obstacles ─────────────────────────────────────
            for obs in &mut obstacles {
                obs.x = obs.x.saturating_sub(1);
            }

            // Remove off-screen obstacles & count score
            if !obstacles.is_empty() && obstacles[0].x == 0 {
                obstacles.remove(0);
                score += 1;
            }

            // Spawn new obstacles with random gap
            if let Some(last) = obstacles.last() {
                let gap = rng.gen_range(MIN_OBSTACLE_GAP..=MAX_OBSTACLE_GAP);
                if last.x <= screen_width.saturating_sub(gap) {
                    let height = match score {
                        0..=4 => rng.gen_range(1..=2),
                        5..=14 => rng.gen_range(1..=3),
                        _ => rng.gen_range(2..=3),
                    };
                    obstacles.push(Obstacle {
                        x: screen_width,
                        height,
                    });
                }
            } else {
                obstacles.push(Obstacle {
                    x: screen_width,
                    height: rng.gen_range(1..=2),
                });
            }

            // ── Collision ──────────────────────────────────────────
            if check_collision(&obstacles, ground_y, player_y, PLAYER_COL) {
                // Game over
                let is_new_best = score > high_score;
                if is_new_best {
                    high_score = score;
                }
                show_game_over(
                    &mut stdout,
                    screen_width,
                    screen_height,
                    score,
                    high_score,
                    is_new_best,
                )?;

                // Wait for restart or quit
                loop {
                    if event::poll(Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Char('q') => break 'outer,
                                KeyCode::Enter | KeyCode::Char(' ') => break,
                                _ => {}
                            }
                        }
                    }
                }
                break; // restart game
            }

            // ── Draw ──────────────────────────────────────────────
            execute!(stdout, terminal::Clear(ClearType::All))?;

            // Score & controls
            draw_score(&mut stdout, score, high_score, screen_width)?;
            draw_controls(&mut stdout, screen_width)?;

            // Elapsed time
            let elapsed = game_start.elapsed().as_secs();
            let time_msg = format!("Time: {}:{:02}", elapsed / 60, elapsed % 60);
            let time_col = (screen_width / 2).saturating_sub(time_msg.len() as u16 / 2);
            execute!(
                stdout,
                cursor::MoveTo(time_col, 1),
                SetForegroundColor(Color::DarkGrey)
            )?;
            write!(stdout, "{}", time_msg)?;
            execute!(stdout, ResetColor)?;

            // Speed indicator
            let speed_pct = ((INITIAL_SPEED_MS as f64 - speed as f64)
                / (INITIAL_SPEED_MS as f64 - 12.0)
                * 100.0)
                .clamp(0.0, 100.0);
            let speed_msg = format!("Speed: {:.0}%", speed_pct);
            execute!(
                stdout,
                cursor::MoveTo(2, 2),
                SetForegroundColor(Color::DarkGrey)
            )?;
            write!(stdout, "{}", speed_msg)?;
            execute!(stdout, ResetColor)?;

            // Ground
            draw_ground(&mut stdout, ground_y, screen_width)?;

            // Obstacles
            for obs in &obstacles {
                draw_obstacle(&mut stdout, obs, ground_y)?;
            }

            // Player
            draw_player(&mut stdout, PLAYER_COL, ground_y, player_y)?;

            stdout.flush()?;
            _frame += 1;
            sleep(Duration::from_millis(speed));
        }
    }

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Thanks for playing Jumpie! High Score: {}", high_score);

    Ok(())
}
