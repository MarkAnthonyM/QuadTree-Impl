const COLOR_PALETTE: [[u8; 4]; 11] = [
    [3, 7, 30, 0],
    [55, 6, 23, 0],
    [106, 4, 15, 0],
    [157, 2, 8, 0],
    [208, 0, 0, 0],
    [220, 47, 2, 0],
    [232, 93, 4, 0],
    [244, 140, 6, 0],
    [250, 163, 7, 0],
    [255, 186, 8, 0],
    [255, 255, 255, 0]
];

struct QuadTree {
    buffer: Vec<usize>,
}

impl QuadTree {
    fn new() -> Self {
        // Setup inital state of buffer with fire igniter at bottom of screen.
        let mut inital_state = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];

        for i in 0..SCREEN_WIDTH {
            inital_state[((SCREEN_HEIGHT - 1) * SCREEN_WIDTH + i) as usize] = COLOR_PALETTE.len() - 1;
        }

        Self { buffer: inital_state }
    }
    
    fn draw(&mut self, screen: &mut [u8]) {
        // Generate iterator broken down in chunks of 4 from window pixel buffer.
        // Equivalent to dividing pixel buffer by 4.
        // E.g: pixel buffer array of length 480,000 becomes of length 120,000
        for (i, pixel) in screen.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&COLOR_PALETTE[self.buffer[i]]);

        }
        
        self.propagate();
    }

    fn propagate(&mut self) {
        for y in 1..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let src = (y * SCREEN_WIDTH + x) as usize;
                let pixel = self.buffer[src];

                if pixel == 0 {
                    self.buffer[src - SCREEN_WIDTH as usize] = 0;
                } else {
                    self.buffer[src - SCREEN_WIDTH as usize] = pixel - 1;
                }
            }
        }
    }
}