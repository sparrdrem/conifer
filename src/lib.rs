use framebuffer::{Framebuffer, KdMode};
use std::fs::OpenOptions;
use std::io::Read;
use std::time::Instant;

const EV_KEY: u32 = 1;
const EV_ABS: u32 = 3;
const EV_MSC: u32 = 4;
const ABS_X: u32 = 0;
const ABS_Y: u32 = 1;
const ABS_MT_SLOT: u32 = 47;
const ABS_MT_POSITION_X: u32 = 53;
const ABS_MT_POSITION_Y: u32 = 54;
const ABS_MT_TRACKING_ID: u32 = 57;
const SYN: u32 = 0;
const BUTTON_LEFT: u32 = 330;

const INPUT_WIDTH: f32 = 719.0;
const INPUT_HEIGHT: f32 = 1439.0;

pub struct Pointer {
    pub is_down: bool,
    pub x: usize,
    pub y: usize,
}

pub struct Frame {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
    line_length: usize,
    bytespp: usize,
}

impl Frame {
    pub fn get_pixel(&mut self, x: usize, y: usize) -> (u8, u8, u8) {
        let curr_index = y * self.line_length + x * self.bytespp;
        (
            self.pixels[curr_index],
            self.pixels[curr_index + 1],
            self.pixels[curr_index + 2],
        )
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let curr_index = y * self.line_length + x * self.bytespp;
        self.pixels[curr_index] = r;
        self.pixels[curr_index + 1] = g;
        self.pixels[curr_index + 2] = b;
    }
}

pub fn run(mut f: impl FnMut(&mut Frame, &Pointer, usize) -> bool) {
    let device = OpenOptions::new()
        .read(true)
        .open("/dev/input/event3")
        .unwrap();
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    let mut pointer = Pointer {
        is_down: false,
        x: 0,
        y: 0,
    };
    let start = Instant::now();
    let mut last_t = 0 as usize;

    let w = framebuffer.var_screen_info.xres as usize;
    let h = framebuffer.var_screen_info.yres as usize;
    let line_length = framebuffer.fix_screen_info.line_length as usize;
    let mut frame = Frame {
        width: w,
        height: h,
        line_length,
        bytespp: (framebuffer.var_screen_info.bits_per_pixel / 8) as usize,
        pixels: vec![0u8; line_length * h],
    };

    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut buffer = [0; 24];

    let t = start.elapsed().as_millis() as usize;
    let delta_t = t - last_t;
    last_t = t;

    let exit = f(&mut frame, &mut pointer, delta_t);
    let _ = framebuffer.write_frame(&frame.pixels);
    if exit {
        return;
    }

    loop {
        let mut b = (&device).take(24).into_inner();
        b.read(&mut buffer);

        let code_a = (buffer[17] as u32) << 8 | buffer[16] as u32;
        let code_b = (buffer[19] as u32) << 8 | buffer[18] as u32;
        let value = (buffer[23] as u32) << 24
            | (buffer[22] as u32) << 16
            | (buffer[21] as u32) << 8
            | (buffer[20] as u32);
        let mut did_update = false;
        if code_a == EV_KEY {
            if code_b == BUTTON_LEFT {
                if value == 1 {
                    pointer.is_down = true;
                    did_update = true;
                } else {
                    pointer.is_down = false;
                }
            }
        } else if code_a == EV_ABS {
            if code_b == ABS_X {
                println!(
                    "{} {} {} {} ",
                    value,
                    INPUT_WIDTH,
                    w,
                    (value as f32 / INPUT_WIDTH * w as f32) as usize
                );
                pointer.x = (value as f32 / INPUT_WIDTH * w as f32) as usize;
            } else if code_b == ABS_Y {
                println!(
                    "{} {} {} {} ",
                    value,
                    INPUT_HEIGHT,
                    h,
                    (value as f32 / INPUT_HEIGHT * h as f32) as usize
                );

                pointer.y = (value as f32 / INPUT_HEIGHT * h as f32) as usize;
            }
        } else if code_a == SYN {
        }

        let t = start.elapsed().as_millis() as usize;
        let delta_t = t - last_t;
        last_t = t;
        let exit = f(&mut frame, &mut pointer, delta_t);
        let _ = framebuffer.write_frame(&frame.pixels);
        if exit {
            break;
        }
    }

    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
