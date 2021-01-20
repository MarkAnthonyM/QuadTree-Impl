use log::{ debug, error };
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::{ LogicalPosition, LogicalSize, PhysicalSize };
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop };
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 400;
const SCREEN_HEIGHT: u32 = 300;

/********************
    QuadTree Logic
********************/

struct QuadTree {}

impl QuadTree {
    fn draw(&mut self, screen: &mut [u8]) {
        // Clear screen
        clear(screen);
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

    let mut quads = QuadTree {};

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            quads.draw(pixels.get_frame());

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
        }
    })
}
