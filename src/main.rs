use log::{ debug, error };
use pixels::{ Error, Pixels, SurfaceTexture };
use std::rc::Rc;
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
        let width = SCREEN_WIDTH as u32;
        let _height = SCREEN_HEIGHT as u8;
        let mut initial_state = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        let circle = Circle::new(20, 50, 1);
        let circle_2 = Circle::new(50, 30, 1);
        let circle_3 = Circle::new(25, 20, 1);
        let circle_4 = Circle::new(90, 40, -1);
        let circle_5 = Circle::new(85, 40, -1);
        let frame_count = 0;

        // initial_state[(circle.coordinates.y * width + circle.coordinates.x) as usize] = circle.color;
        
        SandBox {
            buffer: initial_state,
            circles: vec![circle_2, circle_3],
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
            let mut iter_count = 0;
            for circle in self.circles.iter() {
                let current_coords = (circle.coordinates.x, circle.coordinates.y);
                root = root.insert(current_coords, &mut self.buffer);
                // root.draw(&mut self.buffer);
                println!("{:#?}", root);
                // if iter_count == 1 {
                //     // println!("{:#?}", root);
                //     panic!("this is a test");
                // }
                // iter_count += 1;
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

// struct QuadTree<T> {
//     area: Vec<usize>,
//     point_count: u8,
//     point_limit: u8,
//     nw: Option<T>,
//     ne: Option<T>,
//     sw: Option<T>,
//     se: Option<T>,
// }

// impl<T> QuadTree<T> {
//     fn new() -> Self {
//         QuadTree {
//             area: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
//             point_count: 0,
//             point_limit: 1,
//             nw: None,
//             ne: None,
//             sw: None,
//             se: None,
//         }
//     }

//     fn insert(self) {
//         if self.point_count > 2 {
//             todo!();
//         } else if self.point_count > 0 {
//             todo!();
//         }
//     }

//     fn draw(&mut self, frame: &mut [u8]) {
//         todo!()
//     }
// }

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
        println!("quad insert fired!");
        let quad_location = Quadrant::check_quadrant(data, self.width, self.height);
        // println!("{:?}", quad_location);
        // println!("{:?}", data);
        // println!("{:?}", quad_location);
        // println!("{:?}", self.width);
        //TODO: Fix bug when inserting to node. Try checking children of parent node
        // for a subnode. If found, adjust data coordinates before insertion
        match quad_location {
            Quadrant::Nw => {
                self.nw = Box::new(self.nw.insert(data,buffer));
                // self.nw.draw(buffer);
            },
            Quadrant::Ne => {
                self.ne = Box::new(self.ne.insert(data,buffer));
                // self.ne.draw(buffer);
            },
            Quadrant::Sw => {
                self.sw = Box::new(self.sw.insert(data,buffer));
                // self.sw.draw(buffer);
            },
            Quadrant::Se => {
                self.se = Box::new(self.se.insert(data,buffer));
                // self.se.draw(buffer);
            },
        }

        // self.draw(buffer);
        Branch::Node(self.clone())
    }

    fn draw(&self, buffer: &mut Vec<usize>, location: Quadrant) {
        // Set total height/width of pixel buffer
        let width = self.width * 2;
        let height = self.height * 2;
        // let draw_area = match self.quad_location {
        //     Some(ref quadrant) => {
        //         match quadrant {
        //             Quadrant::Nw => (0, 0),
        //             Quadrant::Ne => (self.width / 2, 0),
        //             Quadrant::Sw => (0, self.height / 2),
        //             Quadrant::Se => (self.width /2, self.height / 2),
        //         }
        //     },
        //     None => {
        //         (0, 0)
        //     }
        // };
        let shift_point = match location {
            Quadrant::Nw => (0, 0),
            Quadrant::Ne => (self.width * 2, 0),
            Quadrant::Sw => (0, self.height),
            Quadrant::Se => (self.width, self.height),
        };
        
        // Fill quadtree cross section
        for y in 0..height {
            for x in 0..width {
                if x == self.width {
                    let src = ((y + shift_point.1) * SCREEN_WIDTH + (x + shift_point.0)) as usize;
                    buffer[src] = 1;
                }

                if y == self.height {
                    let src = ((y + shift_point.1) * SCREEN_WIDTH + (x + shift_point.0)) as usize;
                    buffer[src] = 1;
                }
            }
        }
        // if self.sub_node {
        //     match *self.nw {
        //         Branch::Leaf(ref _asdf) => {
        //             println!("oof");
        //         },
        //         Branch::Node(ref node) => {
        //             node.draw(buffer);
        //         }
        //     }
        // }
    }
}

#[derive(Clone, Debug)]
struct Leaf {
    width: u32,
    height: u32,
    area: u32,
    data_point: Option<(u32, u32)>,
    is_empty: bool,
}

impl Leaf {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            area: width * height,
            data_point: None,
            is_empty: true,
        }
    }

    fn draw(&self) {
        let noooooooo = 0;
    }

    //TODO: Find way to adjust point coordinates based
    // on quadrant point resides in
    // fn insert(&mut self, obj: (u32, u32), buffer: &mut Vec<usize>) -> Branch {
    //     if self.is_empty {
    //         // self.data_point = Some(obj);
    //         // self.is_empty = false;
    //         let mut leaf = Leaf::new(self.width, self.height);
    //         leaf.data_point = Some(obj);
    //         leaf.is_empty = false;

    //         Branch::Leaf(leaf)
    //     } else {
    //         // Split areas
    //         let split_width = self.width / 2;
    //         let split_height = self.height / 2;
    //         // Find previous data's old quadrant
    //         // Adjust previous data's coordiantes based on old quadrant
    //         let prev_data = self.data_point.unwrap();
    //         println!("{:?}", self.width);
    //         let prev_data_updated = match Quadrant::check_quadrant(prev_data, self.width, self.height) {
    //             Quadrant::Nw => prev_data,
    //             Quadrant::Ne => (prev_data.0 - self.width, prev_data.1),
    //             Quadrant::Sw => (prev_data.0, prev_data.1 - self.height),
    //             //BUG: Hitting overflow problem here
    //             Quadrant::Se => (prev_data.0 - self.width, prev_data.1 - self.height),
    //         };
    //         // Adjust new data's coordinates given current parent node
    //         let new_data_updated = match Quadrant::check_quadrant(obj, self.width, self.height) {
    //             Quadrant::Nw => obj,
    //             Quadrant::Ne => (obj.0 - self.width, obj.1),
    //             Quadrant::Sw => (obj.0, obj.1 - self.height),
    //             Quadrant::Se => (obj.0 - self.width, obj.1 - self.height),
    //         };
    //         // Initialze previous and new leaf using previous and new data
    //         let mut prev_leaf = Leaf::new(split_width, split_height);
    //         let mut new_leaf = Leaf::new(split_width, split_height);
    //         // Recursive magic starts here
    //         let processed_prev_leaf = prev_leaf.insert(prev_data_updated, buffer);
    //         let processed_new_leaf = new_leaf.insert(new_data_updated, buffer);

    //         // Generate New node, and store new and previous leaf structs
    //         //TODO: QuadTree node is generating with wrong area. Does it even
    //         // need data about 2D area? Find out.
    //         let mut generate_node = QuadTree::new(split_width, split_height);
    //         //TODO: Should maybe insert into QuadTree node here?
    //         // let mut generate_node = QuadTree::new(self.width, self.height);
    //         // Draw routine
    //         let draw_quad = Quadrant::check_quadrant(obj, generate_node.width, generate_node.height);
    //         let draw_width = generate_node.width;
    //         let draw_height = generate_node.height;
    //         for y in 0..draw_height * 2 {
    //             for x in 0..draw_width * 2 {
    //                 if x == draw_width {
    //                     let src = y * SCREEN_WIDTH + x;
    //                     buffer[src as usize] = 1;
    //                 }

    //                 if y == draw_height {
    //                     let src = y * SCREEN_WIDTH + x;
    //                     buffer[src as usize] = 1;
    //                 }
    //             }
    //         }
    //         // Find new quadrants based on point's adjusted coordinates
    //         let prev_quadrant = Quadrant::check_quadrant(prev_data_updated, split_width, split_height);
    //         let new_quandrant = Quadrant::check_quadrant(new_data_updated, split_width, split_height);
    //         match prev_quadrant {
    //             Quadrant::Nw => {
    //                 generate_node.nw = Box::new(processed_prev_leaf)
    //             },
    //             Quadrant::Ne => {
    //                 generate_node.ne = Box::new(processed_prev_leaf)
    //             },
    //             Quadrant::Sw => {
    //                 generate_node.sw = Box::new(processed_prev_leaf)
    //             },
    //             Quadrant::Se => {
    //                 generate_node.se = Box::new(processed_prev_leaf)
    //             },
    //         }

    //         //TODO: Fix bug in logic here. Logic currently overwrites previous leaf
    //         // if one exists in the same quadrant as the new leaf. Probably need to make use
    //         // of recursive insert logic here?
    //         match new_quandrant {
    //             Quadrant::Nw => {
    //                 // generate_node.nw = Box::new(processed_new_leaf)
    //                 generate_node.nw = Box::new(generate_node.nw.insert(new_data_updated, buffer));
    //                 match *generate_node.nw {
    //                     Branch::Leaf(_) => {},
    //                     Branch::Node(ref node) => {
    //                         let node_location = Quadrant::Nw;
    //                         // node.draw(buffer, node_location);
    //                     }
    //                 }
    //                 // generate_node.quad_location = Some(Quadrant::Nw);
    //             },
    //             Quadrant::Ne => {
    //                 // generate_node.ne = Box::new(processed_new_leaf);
    //                 generate_node.ne = Box::new(generate_node.ne.insert(new_data_updated, buffer));
    //                 match *generate_node.ne {
    //                     Branch::Leaf(_) => {},
    //                     Branch::Node(ref node) => {
    //                         let node_location = Quadrant::Ne;
    //                         // node.draw(buffer, node_location);
    //                     }
    //                 }
    //                 // generate_node.quad_location = Some(Quadrant::Ne);
    //             },
    //             Quadrant::Sw => {
    //                 // generate_node.sw = Box::new(processed_new_leaf)
    //                 generate_node.sw = Box::new(generate_node.sw.insert(new_data_updated, buffer));
    //                 // generate_node.quad_location = Some(Quadrant::Sw);
    //             },
    //             Quadrant::Se => {
    //                 // generate_node.se = Box::new(processed_new_leaf)
    //                 generate_node.se = Box::new(generate_node.se.insert(new_data_updated, buffer));
    //                 // generate_node.quad_location = Some(Quadrant::Se);
    //             },
    //         }
            
    //         // let node_location = Quadrant::Nw;
    //         // generate_node.draw(buffer, node_location);
    //         Branch::Node(generate_node)
    //     }
    // }

    fn insert(&mut self, new_coord: (u32, u32), buffer: &mut Vec<usize>) -> Branch {
        if self.is_empty {
            let mut leaf = Leaf::new(self.width, self.height);
            leaf.data_point = Some(new_coord);
            leaf.is_empty = false;

            Branch::Leaf(leaf)
        } else {
            // Split area
            let split_width = self.width / 2;
            let split_height = self.height / 2;

            // Gather coordinate data of leaf being inserted upon
            let current_coord = self.data_point.unwrap();

            let mut node = QuadTree::new(split_width, split_height);

            //TODO: Need to figure out how to shift sub-node cross section
            // drawing to correct parent quadrant

            // Node draw routine
            for y in 0..self.height {
                for x in 0..self.width {
                    if x == split_width {
                        let src = y * SCREEN_WIDTH + x;
                        buffer[src as usize] = 1;
                    }

                    if y == split_height {
                        let src = y * SCREEN_WIDTH + x;
                        buffer[src as usize] = 1;
                    }
                }
            }

            // Identify node quadrant that current circle resides in and insert
            // into node
            match Quadrant::check_quadrant(current_coord, split_width, split_height) {
                // Adjust current circle coordinates by quadrant width/height.
                // Depending on quadrant circle is found in, width/height will
                // be subtracted from coordinate
                Quadrant::Nw => {
                    let adjusted_coord = current_coord;
                    node.nw = Box::new(node.nw.insert(adjusted_coord, buffer));
                },
                Quadrant::Ne => {
                    let adjusted_coord = (current_coord.0 - split_width, current_coord.1);
                    node.ne = Box::new(node.ne.insert(adjusted_coord, buffer));
                },
                Quadrant::Sw => {
                    let adjusted_coord = (current_coord.0, current_coord.1 - split_height);
                    node.sw = Box::new(node.sw.insert(adjusted_coord, buffer));
                },
                Quadrant::Se => {
                    let adjusted_coord = (current_coord.0 - split_width, current_coord.1 - split_height);
                    node.se = Box::new(node.se.insert(adjusted_coord, buffer));
                }
            }

            // Identify node quadrant that new circle resides in and insert
            // into node
            match Quadrant::check_quadrant(new_coord, split_width, split_height) {
                Quadrant::Nw => {
                // Adjust new circle coordinates by quadrant width/height.
                // Depending on quadrant circle is found in, width/height will
                // be subtracted from coordinate
                    let adjusted_coord = new_coord;
                    node.nw = Box::new(node.nw.insert(adjusted_coord, buffer));
                },
                Quadrant::Ne => {
                    let adjusted_coord = (new_coord.0 - split_width, new_coord.1);
                    node.ne = Box::new(node.ne.insert(adjusted_coord, buffer));
                },
                Quadrant::Sw => {
                    let adjusted_coord = (new_coord.0, new_coord.1 - split_height);
                    node.sw = Box::new(node.sw.insert(adjusted_coord, buffer));
                },
                Quadrant::Se => {
                    let adjusted_coord = (new_coord.0 - split_width, new_coord.1 - split_height);
                    node.se = Box::new(node.se.insert(adjusted_coord, buffer));
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
    fn insert(&mut self, data: (u32, u32), buffer: &mut Vec<usize>) -> Branch {
        match self {
            Branch::Leaf(leaf) => {
                leaf.insert(data, buffer)
            },
            Branch::Node(node) => {
                node.insert(data, buffer)
            },
        }
    }

    fn draw(&self, buffer: &mut Vec<usize>, location: Quadrant) {
        match self {
            Branch::Leaf(leaf) => leaf.draw(),
            Branch::Node(node) => node.draw(buffer, location),
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

fn clear(screen: &mut [u8]) {
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
    let mut paused = false;

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
