use anyhow::{Context, Result};
use std::{io::Read, path::Path};
use tracing::info;

pub struct Shaders {
    pub vertex: wgpu::ShaderModule,
    pub fragment: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new<P: AsRef<Path>>(device: &wgpu::Device, path: P) -> Result<Self> {
        let path = path.as_ref();
        let vertex_src = std::fs::read_to_string(path.join("shader.vert"))
            .context("Couldn't load vertex shader")?;
        let fragment_src = std::fs::read_to_string(path.join("shader.frag"))
            .context("Couldn't load fragment shader")?;
        info!("Finished loading GLSL Shader at: {:?}", path);

        let mut vertex_file =
            glsl_to_spirv::compile(&vertex_src, glsl_to_spirv::ShaderType::Vertex)
                .expect("Couldn't convert vertex shader");
        let mut fragment_file =
            glsl_to_spirv::compile(&fragment_src, glsl_to_spirv::ShaderType::Fragment)
                .expect("Couldn't convert fragment shader");

        let mut vertex_spirv = Vec::new();
        vertex_file.read_to_end(&mut vertex_spirv)?;
        let mut fragment_spirv = Vec::new();
        fragment_file.read_to_end(&mut fragment_spirv)?;

        let vs = wgpu::util::make_spirv(&vertex_spirv);
        let fs = wgpu::util::make_spirv(&fragment_spirv);

        let vertex = device.create_shader_module(vs);
        let fragment = device.create_shader_module(fs);

        info!("Finished compiling SPIR-V Shader at: {:?}", path);

        Ok(Self { vertex, fragment })
    }
}
