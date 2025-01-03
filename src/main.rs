use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    let (screen_width, screen_height) = terminal::size()?;
    let mut player_pos = screen_height;
    let mut obstacles = vec![screen_width];
    let mut score = 0;
    let mut jumping = false;
    let mut velocity = 0;
    let mut speed: u64 = 50; // Initial speed in ms
    let mut rng = rand::thread_rng();

    let mut running = true;
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    loop {
        // Draw score
        execute!(stdout, cursor::MoveTo(0, 0))?;
        print!("Scoreee: {}", score);

        execute!(
            stdout,
            cursor::MoveTo((screen_width / 2) - 15, 5),
            Print("Press 'p' to pause, 'q' to quit")
        )?;

        stdout.flush()?;
        // Handle input
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                } else if key.code == KeyCode::Char('p') {
                    running = !running;
                } else if key.code == KeyCode::Enter && !jumping {
                    jumping = true;
                    velocity = -3;
                }
            }
        }

        if jumping {
            player_pos = (player_pos as i16 + velocity) as u16;
            velocity += 1;
            if player_pos == screen_height {
                jumping = false;
            }
        }

        for obs in &mut obstacles {
            *obs -= 1;
        }

        if obstacles[0] == 0 {
            obstacles.remove(0);
            score += 1;
            speed = speed_up(speed, score);
        }

        if obstacles.last() == Some(&5) {
            obstacles.push(screen_width);
        }

        let gap = rng.gen_range(10..screen_width);
        if obstacles.last() == Some(&gap) {
            obstacles.push(screen_width);
        }

        // Collision detection
        if obstacles.contains(&1)
            && (player_pos == screen_height || player_pos == screen_height - 1)
        {
            break;
        }

        // Draw frame
        execute!(
            stdout,
            cursor::MoveTo(0, screen_height / 2),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        // Draw player
        execute!(stdout, cursor::MoveTo(2, player_pos))?;
        print!("*");

        // Draw obstacles
        for &obs in &obstacles {
            execute!(stdout, cursor::MoveTo(obs, screen_height))?;
            print!("#");
        }

        // Draw score
        execute!(stdout, cursor::MoveTo(0, 0))?;
        print!("Scoreee: {}", score);

        stdout.flush()?;
        sleep(Duration::from_millis(speed));
    }

    // Cleanup
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Game Over! Final Score: {}", score);
    
    Ok(())
}
fn speed_up(speed: u64, score: i32) -> u64 {
    match score {
        score if score > 2 => (speed - 5).max(10),
        score if score > 5 => (speed - 7).max(5),
        score if score > 11 => (speed - 10).max(1),
        score if score > 15 => (speed - 10).max(1),
        _ => speed,
    }
}
