mod config;
mod display;
mod game;
mod input;
mod physics;
mod rendering;
mod sound;
mod spawner;
mod types;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    setup_terminal()?;

    let result = run_game();

    cleanup_terminal()?;
    result
}

fn setup_terminal() -> color_eyre::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::Hide,
        crossterm::terminal::EnterAlternateScreen
    )?;
    Ok(())
}

fn cleanup_terminal() -> color_eyre::Result<()> {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn run_game() -> color_eyre::Result<()> {
    let mut game = game::Game::new()?;

    loop {
        game.show_countdown()?;

        if !game.run()? {
            break;
        }

        if !game.wait_for_restart()? {
            break;
        }
    }

    Ok(())
}
