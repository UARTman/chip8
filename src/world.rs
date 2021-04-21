use crate::cpu::Cpu;
pub struct World {
    pub cpu: Cpu
}


impl World {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new()
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    pub fn update(&mut self) {
        self.cpu.clock_cycle()
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        static WHITE_PIXEL: [u8; 4] = [255; 4];
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let row = i / 64;
            let col = i % 64;
            if self.cpu.vram[col][row] {
                pixel.copy_from_slice(&WHITE_PIXEL)
            } else {
                pixel.copy_from_slice(&[0,0,0,0])
            }
        }
    }
}
