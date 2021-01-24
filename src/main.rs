use log::{ debug, error };
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::{ LogicalPosition, LogicalSize, PhysicalSize };
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop };
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 128;
const SCREEN_HEIGHT: u32 = 128;

/********************
    QuadTree Logic
********************/

struct QuadTree<T> {
    area: Vec<usize>,
    point_count: u8,
    point_limit: u8,
    nw: Option<T>,
    ne: Option<T>,
    sw: Option<T>,
    se: Option<T>,
}

impl<T> QuadTree<T> {
    fn new() -> Self {
        QuadTree {
            area: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
            point_count: 0,
            point_limit: 1,
            nw: None,
            ne: None,
            sw: None,
            se: None,
        }
    }

    fn insert(self) {
        if self.point_count > 2 {
            todo!();
        } else if self.point_count > 0 {
            todo!()
        }
    }

    fn draw(&mut self, frame: &mut [u8]) {
        todo!()
    }
}

/*************************
    Circle Object Logic
*************************/

struct Circle {
    color: [u8; 4],
    coordinates: Point,
    speed: u8,
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

    let mut root = QuadTree::<String>::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            root.draw(pixels.get_frame());

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
            window.request_redraw();
        }
    })
}
