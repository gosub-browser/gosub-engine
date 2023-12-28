use anyhow::Result;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::{CrosstermBackend, Terminal, Frame}, widgets::Paragraph};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Line, Modifier, Stylize};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Tabs, Wrap};
use ratatui::text::{Span, Text};


const HELPTEXT: &'static str = r#"

#1Gosub Dive Help
#1===============
This is the help screen for Gosub Dive. It is a work in progress and displays the current key bindings. This browser is a proof-of-concept project and is not intended for production use.

 #2Function keys
 #2-------------
  #1F1#0      Display this help screen
  #1F2#0
  #1F3#0
  #1F4#0
  #1F5#0
  #1F6#0
  #1F7#0
  #1F8#0
  #1F9#0      Opens top menu

 #2Navigation
 #2----------
  #1CTRL-N#0    Opens new tab with blank page
  #1CTRL-G#0    Asks for an URL to open
  #1CTRL-B#0    Browse back to previous page

 #2General commands
 #2----------------
  #1CTRL-Q#0    Quit Gosub Dive

 #2Tab management
 #2--------------
  #1CTRL-0..9#0 Switch to tab 0..9
  #1TAB#0       Switch to next tab

 #2Navigation
 #2----------
  #1CTRL-N#0    Opens new tab with blank page
  #1CTRL-G#0    Asks for an URL to open
  #1CTRL-B#0    Browse back to previous page

 #2General commands
 #2----------------
  #1CTRL-Q#0    Quit Gosub Dive

 #2Tab management
 #2--------------
  #1CTRL-0..9#0 Switch to tab 0..9
  #1TAB#0       Switch to next tab

"#;

struct Tab {
    name: String,
    url: String,
    content: String,
}

struct App {
    tabs: Vec<Tab>,
    should_quit: bool,
    menu_active: bool,
    menu_item_active: usize,
    current_tab: usize,
    show_help: bool,
    help_scroll: u16,
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn render_help(app: &App, f: &mut Frame) {
    let size = f.size();
    let margins = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10), // Top margin
            Constraint::Percentage(80), // Height of the help block
            Constraint::Percentage(10), // Bottom margin
        ])
        .split(size);

    let help_block_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10), // Left margin
            Constraint::Percentage(80), // Width of the help block
            Constraint::Percentage(10), // Right margin
        ])
        .split(margins[1])[1]; // Select the middle part horizontally within the vertical middle part

    let help_block = Block::default().title(" Help ").borders(Borders::ALL);


    // generate help text, based on #N coloring

    // #0 is default style, #1 is yellow, etc
    let cols = vec![
        Style::default(),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
        Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
    ];

    // This code basically iterates over the lines of the help text. Each line
    // is split into a vector of spans on the #N characters. If a #N is found,
    // the current span is saved, and a new styling for the next span is
    // created. This continues until we have reached the end of the line so
    // each line consists of 1 or more spans with the correct styling. It's
    // then rendered as a paragraph.

    let mut lines = Vec::new();
    let mut partial_line = Vec::new();

    let help_lines = HELPTEXT.split("\n").collect::<Vec<&str>>();
    for line in help_lines {
        let mut cs = Style::default();

        let mut start_idx = 0;
        let mut idx = 0;
        while idx < line.len() {
            let ch = line.chars().nth(idx).unwrap();
            match ch {
                '#' => {
                    if line.chars().nth(idx+1).unwrap().is_ascii_digit() {
                        let line_part: String = line.chars().skip(start_idx).take(idx-start_idx).collect();
                        partial_line.push(Span::styled(line_part, cs));
                        start_idx = idx + 2;

                        let col_idx = line.chars().nth(idx+1).unwrap().to_digit(10).unwrap();
                        cs = cols[col_idx as usize];
                        idx += 2;
                    } else {
                        // Seems like a regular #
                        idx += 1;
                    }
                },
                _ => idx += 1,
            }
        }

        let line_part: String = line.chars().skip(start_idx).take(idx-start_idx).collect();
        partial_line.push(Span::styled(line_part, cs));

        lines.push(Line::from(partial_line.clone()));
        partial_line = Vec::new();
    }

    let text = Text::from(lines);
    let help_paragraph = Paragraph::new(text)
        .block(help_block)
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll, 0))
    ;

    // f.render_widget(help_paragraph, help_block_area);
    f.render_widget(help_paragraph, help_block_area);
}

fn ui(app: &App, f: &mut Frame) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),      // menu bar
                Constraint::Min(0),         // content
                Constraint::Length(1),      // status bar
            ]
                .as_ref(),
        )
        .split(size);

    let mut menu_tiles = vec![
        Span::styled(" Gosub Dive ", Style::default().fg(Color::White).bold()),
        Span::raw("|"),
        Span::raw(" File "),
        Span::raw("|"),
        Span::raw(" Edit "),
        Span::raw("|"),
        Span::raw(" View "),
        Span::raw("|"),
        Span::raw(" History "),
        Span::raw("|"),
        Span::raw(" Bookmarks "),
        Span::raw("|"),
        Span::raw(" Tools "),
        Span::raw("|"),
        Span::raw(" Help "),
    ];

    if app.menu_active {
        menu_tiles[2 + app.menu_item_active * 2] =
            Span::styled(
                menu_tiles[2 + app.menu_item_active * 2].content.clone(),
                Style::default().bg(Color::Green).fg(Color::White).add_modifier(Modifier::BOLD)
            )
        ;
    }

    let menu = Paragraph::new(Line::from(menu_tiles)).style(Style::default().bg(Color::Blue).bold());
    f.render_widget(menu, chunks[0]);

    // let main_area = Block::default().borders(Borders::ALL).title(" gosub://about ");

    let mut tab_names = Vec::new();
    for (idx, tab) in app.tabs.iter().enumerate() {
        tab_names.push(format!(" {}:{} ", idx, tab.name.clone()));
    }

    let tabs = Tabs::new(tab_names)
        .block(Block::default().borders(Borders::NONE))
        // .style(Style::default().white())
        // .highlight_style(Style::default().yellow())
        .select(app.current_tab)
        .divider("|")
        .padding("", "");
    f.render_widget(tabs, chunks[1]);


    let status = Paragraph::new(Line::from(vec![
        Span::styled("Press F1 for help", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::raw("Line 1, Column 1"),
    ])).style(Style::default().bg(Color::Blue).bold());
    f.render_widget(status, chunks[2]);


    if app.show_help {
        render_help(app, f);
    }
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {

            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('0') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 0,
                    Char('1') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 1,
                    Char('2') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 2,
                    Char('3') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 3,
                    Char('4') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 4,
                    Char('5') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 5,
                    Char('6') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 6,
                    Char('7') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 7,
                    Char('8') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 8,
                    Char('9') if key.modifiers.contains(KeyModifiers::CONTROL) => app.current_tab = 9,
                    KeyCode::F(1) => {
                        app.show_help = !app.show_help;
                        app.help_scroll = 0;
                    },

                    KeyCode::Tab => app.current_tab = (app.current_tab + 1 ) % app.tabs.len(),
                    KeyCode::F(9) => app.menu_active = !app.menu_active,

                    Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
                    _ => {},
                }
            }
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let tab1 = Tab {
        name: "New Tab".to_string(),
        url: "gosub://blank".to_string(),
        content: String::new(),
    };

    let tab2 = Tab {
        name: "New Tab".to_string(),
        url: "https://gosub.io".to_string(),
        content: String::new(),
    };

    let mut app = App {
        tabs: vec![tab1, tab2],
        should_quit: false,
        menu_active: false,
        menu_item_active: 0,
        current_tab: 0,
        show_help: false,
        help_scroll: 0,
    };

    loop {
        t.draw(|f| {
            ui(&app, f);
        })?;

        // application update
        update(&mut app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let app = App {
        tabs: vec![],
        should_quit: false,
        menu_active: false,
        menu_item_active: 0,
        current_tab: 0,
        show_help: false,
        help_scroll: 0,
    };

    let backend = ratatui::backend::TestBackend::new(5, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut frame = terminal.get_frame();

    render_help(&app, &mut frame);

    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}