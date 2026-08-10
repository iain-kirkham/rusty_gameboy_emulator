//! Display module wrapping a `minifb` window sized to the Game Boy screen.
//!
//! This is a standalone scaffold for verifying the windowing/rendering path
//! works before it is wired up to real PPU framebuffer output.

use minifb::{Key, Window, WindowOptions};

pub(crate) const SCREEN_WIDTH: usize = 160;
pub(crate) const SCREEN_HEIGHT: usize = 144;

const WINDOW_SCALE: usize = 4;

pub(crate) struct Display {
    window: Window,
    buffer: Vec<u32>,
}

impl Display {
    pub(crate) fn new(title: &str) -> Display {
        let mut window = Window::new(
            title,
            SCREEN_WIDTH * WINDOW_SCALE,
            SCREEN_HEIGHT * WINDOW_SCALE,
            WindowOptions {
                resize: false,
                ..WindowOptions::default()
            },
        )
        .expect("Failed to create window");

        window.set_target_fps(60);

        Display {
            window,
            buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    /// Write a full GB-resolution (160x144) frame of 0xRRGGBB pixels and present it.
    pub(crate) fn present_frame(&mut self, frame: &[u32]) {
        self.buffer.copy_from_slice(frame);
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .expect("Failed to update window buffer");
    }
}

/// Build a simple checkerboard test pattern at GB screen resolution.
fn test_pattern() -> Vec<u32> {
    let mut frame = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let on_light_square = ((x / 8) + (y / 8)) % 2 == 0;
            frame[y * SCREEN_WIDTH + x] = if on_light_square {
                0x00E0_F8D0 // GB-ish light green
            } else {
                0x0034_6856 // GB-ish dark green
            };
        }
    }
    frame
}

/// Open a window and display a static test pattern until the user closes it.
pub(crate) fn run_test_screen() {
    let mut display = Display::new("Rusty Game Boy Emulator - Test Screen");
    let frame = test_pattern();

    while display.is_open() {
        display.present_frame(&frame);
    }
}


