use crate::{render, utils};

use iced_wgpu::{Backend, Renderer, Settings, Viewport};
use iced_winit::{program::State, Debug, Size};
use winit::event::{ModifiersState, WindowEvent};

use utils::MessageChannel;

mod console;
use console::Console;

mod debug;
use debug::DebugInterface;

pub struct InterfaceManager{
    pub renderer: Renderer,
    pub debug: Debug,
    pub viewport: Viewport,
    pub modifiers: ModifiersState,

    pub console_state: State<Console>,
    pub debug_state: State<DebugInterface>,
    pub message_channel: MessageChannel<String>,
}

impl InterfaceManager{
    pub fn new(renderer: &mut render::Renderer) -> Self{
        let window = renderer.window();
        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let modifiers = ModifiersState::default();

        drop(window);
        let device = renderer.device_mut();

        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(device, Settings::default()));

        let controls = Console::new();
        let console_state = State::new(
            controls,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );

        let debug_interface = DebugInterface::new();
        let debug_state = State::new(
            debug_interface,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );

        let message_channel = MessageChannel::new();

        Self{
            renderer,
            debug,
            viewport,
            modifiers,

            console_state,
            debug_state,
            message_channel,
        }
    }

    pub fn update_debug(&mut self, name: &str, value: f64){
        self.debug_state.queue_message(debug::Message::StatChanged((name.to_string(), value)));
    }

    pub fn set_message_channel(&mut self, channel: MessageChannel<String>){
        self.message_channel = channel;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: f64) {
        self.viewport = Viewport::with_physical_size(
            Size::new(new_size.width, new_size.height),
            scale_factor,
        );
    }

    pub fn handle_event(&mut self, event: &WindowEvent, scale_factor: f64){
        if let Some(event) = iced_winit::conversion::window_event(
            event,
            scale_factor,
            self.modifiers
        ){
            self.console_state.queue_event(event);
        }
    }

    pub fn update(&mut self){
        for message in self.message_channel.receiver.try_iter(){
            self.console_state.queue_message(console::Message::NewText(message));
        }

        self.console_state.update(None, self.viewport.logical_size(), &mut self.renderer, &mut self.debug);
        self.debug_state.update(None, self.viewport.logical_size(), &mut self.renderer, &mut self.debug);
    }

    pub fn draw(&mut self, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView){
        self.renderer.backend_mut().draw(
            device,
            encoder,
            view,
            &self.viewport,
            self.debug_state.primitive(),
            &self.debug.overlay(),
        );

        self.renderer.backend_mut().draw(
            device,
            encoder,
            view,
            &self.viewport,
            self.console_state.primitive(),
            &self.debug.overlay(),
        );
    }

    pub fn toggle_console(&mut self){
        self.console_state.queue_message(console::Message::VisibilityChanged);
    }

    pub fn toggle_debug(&mut self){
        self.debug_state.queue_message(debug::Message::VisibilityChanged);
    }

    pub fn console_state(&mut self) -> &mut State<Console>{
        &mut self.console_state
    }

    pub fn renderer(&mut self) -> &mut Renderer{
        &mut self.renderer
    }
}
