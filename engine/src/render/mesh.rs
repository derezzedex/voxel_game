use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex{
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex{
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>{
        use std::mem;
        wgpu::VertexBufferDescriptor{
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ]
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub struct Mesh{
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub size: u32
}

impl Mesh{
    pub fn new(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self{
        let size = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        Self{
            vertex_buffer,
            index_buffer,
            size,
        }
    }
}
