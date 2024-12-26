use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
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
    let mut speed = 50; // Initial speed in ms
    let mut rng = rand::thread_rng();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    loop {
        // Handle input
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
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
            if score == 2 {
                speed = (speed - 5).max(10); // Ensure speed does not go below 10 milliseconds
            }

            if score == 5 {
                speed = (speed - 7).max(5); // Ensure speed does not go below 5 milliseconds
            }

            if score == 11 {
                speed = (speed - 10).max(1); // Ensure speed does not go below 1 milliseconds
            }

            if score == 15 {
                speed = (speed - 15).max(1); // Ensure speed does not go below 1 milliseconds
            }

            if score > 20 {
                speed = (speed - 20).max(1); // Ensure speed does not go below 1 milliseconds
            }
        }

        if obstacles.last() == Some(&5) {
            obstacles.push(screen_width);
        }

        let gap = rng.gen_range(10..screen_width);
        if obstacles.last() == Some(&gap) {
            obstacles.push(screen_width);
        }

        // Collision detection
        if obstacles.contains(&1) && player_pos == screen_height {
            break;
        }

        // Draw frame
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
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
