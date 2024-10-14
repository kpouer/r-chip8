use crate::cpu::Cpu;
use crate::display::Display;

const MAX_MEMORY: usize = 4096;
const OFFSET: usize = 0x200;

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
// https://en.wikipedia.org/wiki/CHIP-8
// https://chip-8.github.io/extensions/
#[derive(Clone, Debug)]
pub(crate) struct Chip8 {
    cpu: Cpu,
    pub(crate) display: Display,
    memory: [u8; MAX_MEMORY],
}

impl Chip8 {
    pub(crate) fn new(rom: Vec<u8>) -> Self {
        let mut memory: [u8; MAX_MEMORY] = [0; MAX_MEMORY];
        for (i, &byte) in rom.iter().enumerate() {
            memory[OFFSET + i] = byte;
        }
        Self {
            cpu: Cpu::new(),
            display: Display::new(),
            memory,
        }
    }

    pub(crate) fn cycle(&mut self) {
        self.cpu.cycle(&mut self.display, &mut self.memory);
    }

    // pub(crate) fn cycle(&mut self) {
    //     let sleep_duration = Duration::from_millis(2);
    //     loop {
    //         self.cpu.cycle(&mut self.display, &self.memory);
    //         thread::sleep(sleep_duration);
    //     }
    // }

    pub(crate) fn should_render(&self) -> bool {
        self.display.is_dirty()
    }
}
