use iced_wgpu::Renderer;
use iced_winit::{
    Container, Column, Row, Text,
    Length, Color, Background,
    Command, Program, Element,
};
use dashmap::DashMap;
use std::time::{Instant, Duration};

#[derive(Debug)]
pub struct DebugInterface{
    visible: bool,

    timer: Instant,
    limiter: Duration,
    stats: DashMap<String, f64>,
    average: DashMap<String, f64>,
}

impl Default for DebugInterface{
    fn default() -> Self{
        Self{
            visible: true,
            timer: Instant::now(),
            limiter: Duration::new(1, 0),
            stats: DashMap::default(),
            average: DashMap::default(),
        }
    }
}

pub type Stat = (String, f64);

#[derive(Debug, Clone)]
pub enum Message {
    VisibilityChanged,
    LimiterChanged(Duration),
    StatChanged(Stat),
}

impl DebugInterface {
    pub fn new() -> Self {
        Self{
            visible: true,
            ..Self::default()
        }
    }
}

impl Program for DebugInterface {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message{
            Message::VisibilityChanged => {
                self.visible = !self.visible;
            },
            Message::LimiterChanged(limiter) =>{
                self.limiter = limiter;
            },
            Message::StatChanged((name, new_stat)) => {
                let stat = if let Some(stat) = self.stats.get(&name){*stat}else{0.} + new_stat;
                self.stats.insert(name, stat);
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        if self.visible{
            if self.timer.elapsed() > self.limiter{
                self.timer = Instant::now();

                self.average.clear();
                for map_ref in self.stats.iter(){
                    self.average.insert(map_ref.key().clone(), map_ref.value().clone());
                }
                self.stats.clear();
            }

            let mut sorted = self.average
                .iter()
                .collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.key().cmp(b.key())); // sorts the debug fields, so that they remain the same every time (Messages are async)
            let column: Element<_, _> = sorted
                .iter()
                .fold(Column::new().spacing(10), |column, map_ref|{
                    let (name, stat) = (map_ref.key(), map_ref.value());
                    let text = if stat.fract() == 0.{
                        format!("{}/s: {}", name, stat)
                    }else{
                        format!("{}/s: {:.2}", name, stat)
                    };
                    column.push(
                        Text::new(text).size(12)
                    )
                })
                .into();
            Container::new(column)
                .padding(5)
                .style(ContainerStyle)
                .into()
        }else{
            Column::new().into()
        }
    }
}

use iced::container;
struct ContainerStyle;

impl container::StyleSheet for ContainerStyle{
    fn style(&self) -> container::Style{
        container::Style {
            background: Some(Background::Color(Color::from_rgba8(
                0x36, 0x39, 0x3F, 0.75
            ))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
