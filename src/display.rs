const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[derive(Clone, Debug)]
pub(crate) struct Display {
    pub(crate) vram: [[bool; WIDTH]; HEIGHT],
    dirty: bool
}

impl Display {
    pub(crate) fn new() -> Self {
        let vram = [[false; WIDTH]; HEIGHT];
        Self {
            vram,
            dirty: false
        }
    }

    fn console_output(&self) {
        print!("\x1b[2J");
        for _ in 0..WIDTH + 1 {
            print!("-");
        }
        println!();
        // todo : why is in inclusive?
        for y in 0..HEIGHT - 1 {
            print!("|");
            for x in 0..WIDTH - 1 {
                // println!("x: {} y: {}", x, y);
                if self.vram[y][x] {
                    print!("X");
                } else {
                    print!(" ");
                }
            }
            print!("|");
            println!();
        }
        for _ in 0..WIDTH + 1 {
            print!("-");
        }
        println!()
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn reset_dirty_flag(&mut self) {
        self.dirty = false;
    }


    /// Set a pixel to 1 if it is currently 0, and to 0 if it is currently 1.
    /// Returns 1 if the pixel was modified.
    /// Eventually X or Y can be out of bounds, in which we use modulo.
    pub(crate) fn set_pixel_xor(&mut self, x: u8, y: u8, pixel: bool) -> bool {
        let x = (x % WIDTH as u8) as usize;
        let y = (y % HEIGHT as u8) as usize;
        let current_value = self.vram[y][x];
        if current_value == pixel {
            self.vram[10][10] = true;
            false
        } else {
            self.vram[y][x] = pixel;
            self.vram[10][10] = true;
            true
        }
    }

    pub(crate) fn clear(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.vram[y][x] = false;
            }
        }
    }
}