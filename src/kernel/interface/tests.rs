use super::{
    Cell, Color, MAX_COLUMNS, MAX_ROWS, Surface, ansi::AnsiSurface, input::Key, window::Desktop,
};

struct Screen([Cell; MAX_COLUMNS * MAX_ROWS]);

impl Surface for Screen {
    fn draw_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.0[y * MAX_COLUMNS + x] = cell;
    }
}

#[test]
fn windows_fill_the_desktop_interior_at_every_supported_size() {
    for rows in 7..=MAX_ROWS {
        for columns in 14..=MAX_COLUMNS {
            let mut desktop = Desktop::new();
            assert!(desktop.init(columns, rows));
            desktop.key(Key::Resize);
            for _ in 0..MAX_COLUMNS {
                desktop.key(Key::Right);
                desktop.key(Key::Down);
            }
            let bounds = desktop.bounds();
            assert_eq!(bounds.x + bounds.width, columns - 1);
            assert_eq!(bounds.y + bounds.height, rows - 1);

            let mut screen = Screen([Cell::EMPTY; MAX_COLUMNS * MAX_ROWS]);
            desktop.render(&mut screen);
            assert_eq!(
                screen.0[(rows - 2) * MAX_COLUMNS + 1],
                Cell::new('┗', Color::Green)
            );
            assert_eq!(
                screen.0[(rows - 1) * MAX_COLUMNS + 1],
                Cell::new('━', Color::Blue)
            );
        }
    }
}

#[test]
fn cursor_stays_inside_content_during_shrink_and_grow() {
    let mut desktop = Desktop::new();
    assert!(desktop.init(MAX_COLUMNS, MAX_ROWS));
    for _ in 0..40 {
        for &byte in b"A long line of terminal output that wraps when the window shrinks.\n" {
            desktop.terminal.putc(byte);
        }
    }
    desktop.key(Key::Resize);
    for direction in [Key::Up, Key::Left, Key::Down, Key::Right] {
        for _ in 0..MAX_COLUMNS {
            desktop.key(direction);
            let bounds = desktop.bounds();
            let (x, y) = desktop.terminal.cursor();
            assert!(x < bounds.width - 2);
            assert!(y < bounds.height - 2);
        }
    }
    assert_eq!(desktop.bounds().y + desktop.bounds().height, MAX_ROWS - 1);
}

#[test]
fn repaint_hides_cursor_until_input_position_is_restored() {
    let mut desktop = Desktop::new();
    assert!(desktop.init(MAX_COLUMNS, MAX_ROWS));
    for key in [
        Key::Resize,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Resize,
    ] {
        desktop.key(key);
        let mut output = String::new();
        let bounds = desktop.bounds();
        let (x, y) = desktop.terminal.cursor();
        let mut surface = AnsiSurface::new(&mut output);
        desktop.render(&mut surface);
        surface.finish(bounds.x + 1 + x, bounds.y + 1 + y);
        assert!(output.starts_with("\x1b[?25l"));
        assert_eq!(output.matches("\x1b[?25l").count(), 1);
        assert_eq!(output.matches("\x1b[?25h").count(), 1);
        assert!(output.ends_with(&format!(
            "\x1b[0m\x1b[{};{}H\x1b[?25h",
            bounds.y + 2 + y,
            bounds.x + 2 + x,
        )));
    }

    let mut output = String::new();
    let mut surface = AnsiSurface::new(&mut output);
    desktop.render(&mut surface);
    surface.finish(2, 2);
    assert_eq!(output, "\x1b[0m\x1b[3;3H");
}
