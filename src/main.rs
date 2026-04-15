use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use std::time::Duration;
use std::process::Command; 
use ansi_to_tui::IntoText; // need this for the weather curl
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    running: bool, /// Is the application running?
    username: String, // self explainatory, uses whoami crate
    current_time: String,
    weather: String,
    fortune: String,
    city: String,
    status: String,
}




impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self {
            running: false,
            username: whoami::username().unwrap_or_else(|_| "User".to_string()),
            current_time: "Shot-o-Clock".to_string(),
            weather: "idk yet lol".to_string(), 
            fortune: "loading up something whitty...".to_string(),
            city: "Athens".to_string(), // default to Athens for now
            status: "Booting ts up!".to_string()
        };
        app.refresh_weather();
        app.refresh_fortune();
        app.update_time();
        app
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            self.update_time();
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Ryan\'s RataTUI")
            .bold()
            .red() // .on_white() is so tuff
            .centered();

        let areas = Layout::vertical([
            Constraint::Length(3), // header
            Constraint::Fill(1), // body
            Constraint::Length(3), // footer
        ])
        .split(frame.area()); 

        let body_text = format!(
            "Hello, {}!\n\
    Created using https://github.com/ratatui/templates", self.username);

        frame.render_widget(
            Paragraph::new("")
                .block(Block::bordered().title(title)),
            areas[0],
        );
        let body_chunks = Layout::vertical([
    Constraint::Length(3), // info row
    Constraint::Fill(1),   // main content
])
.split(areas[1]);

// split bottom into weather + fortune
let bottom_chunks = Layout::horizontal([
    Constraint::Percentage(70),
    Constraint::Percentage(30),
])
.split(body_chunks[1]);

let info_text = format!(
    "User: {}    |    Time: {}",
    self.username, self.current_time
);

frame.render_widget(
    Paragraph::new(info_text)
        .block(Block::bordered().title("Info")),
    body_chunks[0],
);
let weather_text: Text = self.weather.into_text().unwrap_or_default().into();

frame.render_widget(
    Paragraph::new(weather_text)
        .block(Block::bordered().title(format!("Weather ({})", self.city))),
    bottom_chunks[0],
);
frame.render_widget(
    Paragraph::new(self.fortune.clone())
        .block(Block::bordered().title("Fortune")),
    bottom_chunks[1],
);

// show status changes in footer! (Where they belong tbh)

        frame.render_widget(
            Paragraph::new(format!("q / ctrl+c / Esc: quit   |   r: new fortune   |   {}", self.status)).block(Block::bordered()),
            areas[2],
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        
        Ok(())
    }


    fn refresh_weather(&mut self) {
        let url = format!("wttr.in/{}?u", self.city);
        match Command::new("curl").arg("-s").arg(&url).output() { // so the -s is to make sure the next arg is one cohesive string (i think),
            //                                 ^^ I have no idea why a & is necessary here but..
            Ok(output) if output.status.success() => {
                self.weather = String::from_utf8_lossy(&output.stdout).to_string();
                self.status = "Weather Loaded".to_string();
            }
            // the below cases are for error stuff
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                self.weather = "Failed to load weather".to_string();
                self.status = format!("Error loading weather: {}", err.trim());
            }
            Err(e) => { // so u can let the compiler know that you wont use an error value by just defining it as _ (underscore)
                self.weather = "Failed to load weather is curl installed?".to_string();
                self.status = format!("Error executing curl: {}", e);
            }
        }
    }
    fn refresh_fortune(&mut self) {
        match Command::new("sh").arg("-c").arg("fortune | cowsay -r").output() {
            Ok(output) if output.status.success() => {
                self.fortune = String::from_utf8_lossy(&output.stdout).to_string();
                self.status = "Fortune Loaded".to_string();
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                self.fortune = "Fortune unavailable. Is fortune and cowsay installed?".to_string();
                self.status = format!("Fortune and/or cowsay failed: {}", err.trim());
            }
            Err(e) => {
                self.fortune = "Fortune unavailable. Is fortune and cowsay installed?".to_string();
                self.status = format!("Error executing fortune/cowsay: {}", e);
            }
        }
    }
    fn update_time(&mut self) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    let secs = now.as_secs() % 86400;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    self.current_time = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}



    /// Handles the key events and updates the state of [`App`]. Quitting app and refreshing fortunes
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Char('r')) => {
                self.status = "Refreshing...".to_string();
                self.refresh_fortune();
                self.refresh_weather();
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
