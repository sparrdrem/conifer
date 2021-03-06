fn main() {
    conifer::run(|frame, pointer, delta_time| {
        for y in 0..frame.height {
            for x in 0..frame.width {
                if pointer.is_down {
                    return true;
                }
                frame.set_pixel(
                    x,
                    y,
                    ((x as f32 / frame.width as f32) * 255.0) as u8,
                    ((y as f32 / frame.height as f32) * 255.0) as u8,
                    0 as u8,
                );
            }
        }
        false
    })
}
