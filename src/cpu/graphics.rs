impl super::Cpu {
    fn draw_pixel(&mut self, mut x: usize, y: usize, px: bool) -> bool {
        x %= 64;
        let res = self.vram[x][y] && px;
        self.vram[x][y] ^= px;
        res
    }

    fn draw_line(&mut self, x: usize, y: usize, line: u8) -> bool {
        Decon::new(line)
            .enumerate()
            .map(|(idx, px)| self.draw_pixel(x + idx, y % 32, px))
            .any(|y| y)
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        sprite
            .iter()
            .enumerate()
            .map(|(idx, &line)| self.draw_line(x, y + idx, line))
            .any(|y| y)
    }
}

struct Decon(u8, u8);

impl Decon {
    fn new(x: u8) -> Self {
        Self(x, 0)
    }
}

impl Iterator for Decon {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 7 {
            None
        } else {
            let x = (self.0 >> (7 - self.1)) & 1;
            self.1 += 1;
            Some(x == 1)
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_decon() {
        let a: u8 = 0b10101010;
        let d = super::Decon::new(a);
        let col: Vec<bool> = d.collect();
        assert_eq!(col, vec![true, false, true, false, true, false, true, false]);
    }
}
