use std::{path::Path, io::Read};
use anyhow::{Context, Result};
use log::info;

pub struct Shaders{
    pub vertex: wgpu::ShaderModule,
    pub fragment: wgpu::ShaderModule,
}

impl Shaders{
    pub fn new<P: AsRef<Path>>(device: &wgpu::Device, path: P) -> Result<Self>{
        let path = path.as_ref();
        let vertex_src = std::fs::read_to_string(path.join("shader.vert")).context("Couldn't load vertex shader")?;
        let fragment_src = std::fs::read_to_string(path.join("shader.frag")).context("Couldn't load fragment shader")?;
        info!("Finished loading GLSL Shader at: {:?}", path);

        let mut vertex_file = glsl_to_spirv::compile(&vertex_src, glsl_to_spirv::ShaderType::Vertex).expect("Couldn't convert vertex shader");
        let mut fragment_file = glsl_to_spirv::compile(&fragment_src, glsl_to_spirv::ShaderType::Fragment).expect("Couldn't convert fragment shader");
        
        let mut vertex_spirv = Vec::new();
        vertex_file.read_to_end(&mut vertex_spirv)?;
        let mut fragment_spirv = Vec::new();
        fragment_file.read_to_end(&mut fragment_spirv)?;
        // let mut compiler = shaderc::Compiler::new().context("Couldn't create shaderc compiler")?;
        // let vertex_spirv = compiler.compile_into_spirv(
        //     &vertex_src, shaderc::ShaderKind::Vertex,
        //     "shader.vert", "main", None).context("Couldn't convert GLSL to SPIR-V")?;

        // let fragment_spirv = compiler.compile_into_spirv(
        //     &fragment_src, shaderc::ShaderKind::Fragment,
        //     "shader.frag", "main", None).context("Couldn't convert GLSL to SPIR-V")?;

        let vs = wgpu::util::make_spirv(&vertex_spirv);
        let fs = wgpu::util::make_spirv(&fragment_spirv);

        let vertex = device.create_shader_module(vs);
        let fragment = device.create_shader_module(fs);

        info!("Finished compiling SPIR-V Shader at: {:?}", path);

        Ok(Self{
            vertex,
            fragment,
        })
    }

    pub fn from_spirv<P: AsRef<Path>>(device: &wgpu::Device, path: P) -> Result<Self, anyhow::Error>{
        let path = path.as_ref();
        let vertex_spirv = std::fs::read(path.join("frag.spv")).context("Couldn't load vertex shader")?;
        let fragment_spirv = std::fs::read(path.join("vert.spv")).context("Couldn't load fragment shader")?;
        //
        // // let vertex_spirv = glsl_to_spirv::compile(&vertex_src, glsl_to_spirv::ShaderType::Vertex).context("Couldn't convert vertex shader");
        // // let fragment_spirv = glsl_to_spirv::compile(&fragment_src, glsl_to_spirv::ShaderType::Fragment).context("Couldn't convert fragment shader");
        // let mut compiler = shaderc::Compiler::new().context("Couldn't create shaderc compiler")?;
        // let vertex_spirv = compiler.compile_into_spirv(
        //     &vertex_src, shaderc::ShaderKind::Vertex,
        //     "shader.vert", "main", None).context("Couldn't convert GLSL to SPIR-V")?;
        //
        // let fragment_spirv = compiler.compile_into_spirv(
        //     &fragment_src, shaderc::ShaderKind::Fragment,
        //     "shader.frag", "main", None).context("Couldn't convert GLSL to SPIR-V")?;

        let vs = wgpu::util::make_spirv(&vertex_spirv);
        let fs = wgpu::util::make_spirv(&fragment_spirv);

        let vertex = device.create_shader_module(vs);
        let fragment = device.create_shader_module(fs);

        info!("Finished loading SPIR-V Shader at: {:?}", path);

        Ok(Self{
            vertex,
            fragment,
        })
    }
}
