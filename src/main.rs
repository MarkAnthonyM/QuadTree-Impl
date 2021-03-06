use log::{ debug, error };
use pixels::{ Error, Pixels, SurfaceTexture };
use std::{ thread, time };
use winit::dpi::{ LogicalPosition, LogicalSize, PhysicalSize };
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop };
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 128;
const SCREEN_HEIGHT: u32 = 128;

const PALETTE: [[u8; 4]; 2] = [
    [255, 255, 255, 0],
    [0, 0, 0, 0],
];

struct SandBox {
    buffer: Vec<usize>,
    circles: Vec<Circle>,
    frame_count: u32,
}

impl SandBox {
    fn new() -> Self {
        let _width = SCREEN_WIDTH as u32;
        let _height = SCREEN_HEIGHT as u8;
        let initial_state = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let circle = Circle::new(20, 50, 1);
        let circle_2 = Circle::new(50, 30, 1);
        let circle_3 = Circle::new(25, 20, 1);
        let circle_4 = Circle::new(90, 40, -1);
        let circle_5 = Circle::new(85, 40, -1);
        let frame_count = 0;
        
        SandBox {
            buffer: initial_state,
            circles: vec![circle, circle_2, circle_3, circle_4, circle_5],
            frame_count,
        }
    }

    fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = 0;
        }
    }

    fn update(&mut self) {
        self.clear();

        // Increment frame count on every draw
        self.frame_count += 1;

        // Step circle pixel back and forth between quadrants
        let amplitude = 3.0;
        let two_pi = 2.0 * std::f64::consts::PI;
        let period = 100.0;
        let frame_count = self.frame_count as f64;
        let oscillation = (amplitude * f64::sin(two_pi * (frame_count / period))) as i32;

        // Update position of circles based on oscillation calculation
        for circle in self.circles.iter_mut() {
            let current_x = if circle.direction > 0 {
                circle.coordinates.x as i32 + oscillation
            } else {
                circle.coordinates.x as i32 - oscillation
            };
            circle.coordinates.x = current_x as u32;
            
            let src = (circle.coordinates.y * SCREEN_WIDTH + circle.coordinates.x) as usize;
            self.buffer[src] = circle.color;
        }

        if !self.circles.is_empty() {
            let leaf = Leaf::new(SCREEN_WIDTH, SCREEN_HEIGHT);
            let mut root = Branch::Leaf(leaf);
            for circle in self.circles.iter() {
                let current_coords = (circle.coordinates.x, circle.coordinates.y);
                root = root.insert(current_coords, None, None, &mut self.buffer);
            }
        }

        let one_sec = time::Duration::from_millis(300);
        thread::sleep(one_sec);
    }

    fn draw(&mut self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&PALETTE[self.buffer[i]]);
        }
    }
}

/********************
    QuadTree Logic
********************/

#[derive(Clone, Debug)]
struct QuadTree {
    area: u32,
    width: u32,
    height: u32,
    point_count: u8,
    point_limit: u8,
    nw: Box<Branch>,
    ne: Box<Branch>,
    sw: Box<Branch>,
    se: Box<Branch>,
    quad_location: Option<Quadrant>,
}

impl QuadTree {
    fn new(width: u32, height: u32) -> Self {
        QuadTree {
            area: width * height,
            width,
            height,
            point_count: 0,
            point_limit: 1,
            nw: Box::new(Branch::Leaf(Leaf::new(width, height))),
            ne: Box::new(Branch::Leaf(Leaf::new(width, height))),
            sw: Box::new(Branch::Leaf(Leaf::new(width, height))),
            se: Box::new(Branch::Leaf(Leaf::new(width, height))),
            quad_location: None,
        }
    }

    fn insert(&mut self, data: (u32, u32), buffer: &mut Vec<usize>) -> Branch {
        // Check new data coordinate for residing quadrant
        let quad_location = Quadrant::check_quadrant(data, self.width, self.height);

        // Match against data quadrant location. Adjusted coordinate data used for
        // placing data points into proper nodes/sub nodes
        // quadrant variable used for drawing routine
        match quad_location {
            Quadrant::Nw => {
                let adjusted_coord = data;
                let quadrant = Quadrant::Nw;
                self.nw = Box::new(self.nw.insert(data, Some(adjusted_coord), Some(quadrant), buffer));
            },
            Quadrant::Ne => {
                let adjusted_coord = (data.0 - self.width, data.1);
                let quadrant = Quadrant::Ne;
                self.ne = Box::new(self.ne.insert(data, Some(adjusted_coord), Some(quadrant), buffer));
            },
            Quadrant::Sw => {
                let adjusted_coord = (data.0, data.1 - self.height);
                let quadrant = Quadrant::Sw;
                self.sw = Box::new(self.sw.insert(data, Some(adjusted_coord), Some(quadrant), buffer));
            },
            Quadrant::Se => {
                let adjusted_coord = (data.0 - self.width, data.1 - self.height);
                let quadrant = Quadrant::Se;
                self.se = Box::new(self.se.insert(data, Some(adjusted_coord), Some(quadrant), buffer));
            },
        }

        Branch::Node(self.clone())
    }
}

#[derive(Clone, Debug)]
struct Leaf {
    width: u32,
    height: u32,
    area: u32,
    adjusted_data_point: Option<(u32, u32)>,
    data_point: Option<(u32, u32)>,
    is_empty: bool,
}

impl Leaf {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            area: width * height,
            adjusted_data_point: None,
            data_point: None,
            is_empty: true,
        }
    }

    fn insert(&mut self, new_coord: (u32, u32), adjusted_data: Option<(u32, u32)>, quadrant: Option<Quadrant>, buffer: &mut Vec<usize>) -> Branch {
        if self.is_empty {
            let mut leaf = Leaf::new(self.width, self.height);
            if adjusted_data.is_some() {
                leaf.adjusted_data_point = adjusted_data;
            }
            leaf.data_point = Some(new_coord);
            leaf.is_empty = false;

            Branch::Leaf(leaf)
        } else {
            // Split area
            let split_width = self.width / 2;
            let split_height = self.height / 2;

            // Gather original coordinate data of leaf being inserted upon
            let original_coord = self.data_point.unwrap();

            // Gather adjusted data for current coordinates, if any
            let current_coord = if let Some(coord) = self.adjusted_data_point {
                coord
            } else {
                self.data_point.unwrap()
            };

            // Gather adjusted data for new coordinates, if any
            let new_coord_adjusted = if let Some(coord) = adjusted_data {
                coord
            } else {
                new_coord
            };

            let mut node = QuadTree::new(split_width, split_height);

            //TODO: Need to figure out how to shift sub-node cross section
            // drawing to correct parent quadrant
            let shift_drawing = match quadrant {
                Some(area) => {
                    match area {
                        Quadrant::Nw => (0, 0),
                        Quadrant::Ne => (self.width, 0),
                        Quadrant::Sw => (0, self.height),
                        Quadrant::Se => (self.width, self.height),
                    }
                },
                None => (0, 0),
            };

            // Node draw routine
            for y in 0..self.height {
                for x in 0..self.width {
                    if x == split_width {
                        let src = (y + shift_drawing.1) * SCREEN_WIDTH + (x + shift_drawing.0);
                        buffer[src as usize] = 1;
                    }

                    if y == split_height {
                        let src = (y + shift_drawing.1) * SCREEN_WIDTH + (x + shift_drawing.0);
                        buffer[src as usize] = 1;
                    }
                }
            }

            // Identify node quadrant that current circle resides in and insert
            // into node
            match Quadrant::check_quadrant(current_coord, split_width, split_height) {
                // Adjust current circle coordinates by quadrant width/height.
                // Depending on quadrant circle is found in, width/height will
                // be subtracted from coordinate.
                // Adjusted coordinate data used to place object into
                // proper node/sub node
                Quadrant::Nw => {
                    let adjusted_coord = current_coord;
                    node.nw = Box::new(node.nw.insert(original_coord, Some(adjusted_coord), None, buffer));
                },
                Quadrant::Ne => {
                    let adjusted_coord = (current_coord.0 - split_width, current_coord.1);
                    node.ne = Box::new(node.ne.insert(original_coord, Some(adjusted_coord), None, buffer));
                },
                Quadrant::Sw => {
                    let adjusted_coord = (current_coord.0, current_coord.1 - split_height);
                    node.sw = Box::new(node.sw.insert(original_coord, Some(adjusted_coord), None, buffer));
                },
                Quadrant::Se => {
                    let adjusted_coord = (current_coord.0 - split_width, current_coord.1 - split_height);
                    node.se = Box::new(node.se.insert(original_coord, Some(adjusted_coord), None, buffer));
                }
            }

            // Identify node quadrant that new circle resides in and insert
            // into node
            match Quadrant::check_quadrant(new_coord_adjusted, split_width, split_height) {
                // Adjust new circle coordinates by quadrant width/height.
                // Depending on quadrant circle is found in, width/height will
                // be subtracted from coordinate.
                // Adjusted coordinate data used to place object into
                // proper node/sub node
                Quadrant::Nw => {
                    let adjusted_coord = new_coord_adjusted;
                    let quadrant = Quadrant::Nw;
                    node.nw = Box::new(node.nw.insert(new_coord, Some(adjusted_coord), Some(quadrant), buffer));
                },
                Quadrant::Ne => {
                    let adjusted_coord = (new_coord_adjusted.0 - split_width, new_coord_adjusted.1);
                    let quadrant = Quadrant::Ne;
                    node.ne = Box::new(node.ne.insert(new_coord, Some(adjusted_coord), Some(quadrant), buffer));
                },
                Quadrant::Sw => {
                    let adjusted_coord = (new_coord_adjusted.0, new_coord_adjusted.1 - split_height);
                    let quadrant = Quadrant::Sw;
                    node.sw = Box::new(node.sw.insert(new_coord, Some(adjusted_coord), Some(quadrant), buffer));
                },
                Quadrant::Se => {
                    let adjusted_coord = (new_coord_adjusted.0 - split_width, new_coord_adjusted.1 - split_height);
                    let quadrant = Quadrant::Se;
                    node.se = Box::new(node.se.insert(new_coord, Some(adjusted_coord), Some(quadrant), buffer));
                }
            }

            Branch::Node(node)
        }
    }
}

#[derive(Clone, Debug)]
enum Branch {
    Node(QuadTree),
    Leaf(Leaf),
}

impl Branch {
    fn insert(&mut self, data: (u32, u32), adjusted_data: Option<(u32, u32)>, quadrant: Option<Quadrant>, buffer: &mut Vec<usize>) -> Branch {
        match self {
            Branch::Leaf(leaf) => {
                leaf.insert(data, adjusted_data, quadrant, buffer)
            },
            Branch::Node(node) => {
                node.insert(data, buffer)
            },
        }
    }
}

/*************************
    Circle Object Logic
*************************/

struct Circle {
    color: usize,
    coordinates: Point,
    direction: i32,
    speed: u32,
}

impl Circle {
    fn new(x: u32, y: u32, direction: i32) -> Self {
        Circle {
            color: 1,
            coordinates: Point::new(x, y),
            direction,
            speed: 1,
        }
    }
}

#[derive(Clone, Debug)]
enum Quadrant {
    Nw,
    Ne,
    Sw,
    Se,
}

impl Quadrant {
    // May need to change how quadrant limits are checked in order
    // to fix issues with drawing quadrant cross-section graphics
    fn check_quadrant(obj_coord: (u32, u32), width: u32, height: u32) -> Quadrant {
        let (current_x, current_y) = (obj_coord.0, obj_coord.1);
        if current_x <= width && current_y <= height {
            return Quadrant::Nw
        } else if current_x >= width && current_y <= height {
            return Quadrant::Ne
        } else if current_x <= width && current_y >= height {
            return Quadrant::Sw
        } else {
            return Quadrant::Se
        };
    }
}

struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Self {
        Point {
            x,
            y,
        }
    }
}

fn create_window(
    title: &str,
    event_loop: &EventLoop<()>
) -> (winit::window::Window, u32, u32, f64) {
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

    let min_size: winit::dpi::LogicalSize<f64> = PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (window, size.width.round() as u32, size.height.round() as u32, hidpi_factor)
}

fn _clear(screen: &mut [u8]) {
    for (i, byte) in screen.iter_mut().enumerate() {
        *byte = if i % 4 == 3 { 255 } else { 0 };
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, p_width, p_height, mut _hidpi_factor) =
        create_window("This is a test", &event_loop);
    
    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
    
    let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?;
    let mut _paused = false;

    let mut draw_state: Option<bool> = None;

    let mut sand_box = SandBox::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            sand_box.draw(pixels.get_frame());

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
        }
        
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let (mouse_cell, mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    // Mouse coordinates of last event step
                    let (dx, dy) = input.mouse_diff();
                    // Gather previous xy coordinates by subtracting current mouse
                    // coordinates from last event step's coordinates
                    let prev_x = mx - dx;
                    let prev_y = my - dy;

                    // Index that mouse currently resides in
                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    
                    // Index that mouse previously residing in
                    let (px_i, py_i) = pixels
                        .window_pos_to_pixel((prev_x, prev_y))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                    
                    ((mx_i as isize, my_i as isize), (px_i as isize, py_i as isize))
                })
                .unwrap_or_default();
            
            if input.mouse_pressed(0) {
                debug!("Mouse clicked at {:?}", mouse_cell);
                draw_state = Some(true);
            } else if let Some(draw_alive) = draw_state {
                let release = input.mouse_released(0);
                let held = input.mouse_held(0);
                debug!("Draw at {:?} => {:?}", mouse_prev_cell, mouse_cell);
                debug!("Mouse held {:?}, release {:?}", held, release);

                if release || held {
                    debug!("Draw line of {:?}", draw_alive);
                }

                if release || !held {
                    debug!("Draw end");
                    draw_state = None;
                }
            }

            sand_box.update();
            window.request_redraw();
        }
    })
}
