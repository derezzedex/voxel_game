use winit::{
    window::{WindowBuilder, Window},
    event_loop::EventLoop,
};
use log::{info, error};

pub struct Renderer{
    window: Window,

    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

impl Renderer{
    pub async fn new(event_loop: &EventLoop<()>) -> Self{
        let window = WindowBuilder::new()
            .with_title("Voxel game")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .expect("Couldn't create window");

        if let Err(e) = window.set_cursor_grab(true){
            error!("Couldn't grab mouse, error: {}", e);
        }

        window.set_cursor_visible(false);
        window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));

        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Couldn't request the Adapter");

        info!("{:#?}", adapter.get_info());

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            extensions: wgpu::Extensions{
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        }).await;

        let size = window.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor{
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self{
            window,

            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }

    pub fn get_window(&self) -> &Window{
        &self.window
    }
}
