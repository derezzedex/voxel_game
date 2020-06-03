use image::GenericImageView;
use std::path::Path;
use anyhow::{Context, Result};
use log::info;

pub struct Texture{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture{
    pub fn from_path<P: AsRef<Path>>(device: &wgpu::Device, path: P) -> Result<(Self, wgpu::CommandBuffer), anyhow::Error>{
        info!("Loading texture from file: {:?}", path.as_ref());
        let img_bytes = std::fs::read(path).context("Couldn't load file")?;
        Texture::from_bytes(&device, &img_bytes)
    }

    pub fn from_bytes(device: &wgpu::Device, bytes: &[u8]) -> Result<(Self, wgpu::CommandBuffer), anyhow::Error>{
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, &img)
    }

    pub fn from_image(device: &wgpu::Device, img: &image::DynamicImage)  -> Result<(Self, wgpu::CommandBuffer), anyhow::Error> {
        let rgba = img.as_rgba8().context("Couldn't load as RGBA8")?;
        let dim = img.dimensions();

        let tex_size = wgpu::Extent3d {
            width: dim.0,
            height: dim.1,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor{
            label: Some("Tree tex"),
            size: tex_size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let buffer = device.create_buffer_with_data(
            &rgba,
            wgpu::BufferUsage::COPY_SRC,
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("texture buffer copy encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView{
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * dim.0,
                rows_per_image: dim.1,
            },
            wgpu::TextureCopyView{
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            tex_size,
        );

        let cmd_buffer = encoder.finish();

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Ok((Self { texture, view, sampler }, cmd_buffer))
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, label: &str) -> Self{
        let size = wgpu::Extent3d{
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let desc = wgpu::TextureDescriptor{
            label: Some(label),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                |  wgpu::TextureUsage::SAMPLED
                |  wgpu::TextureUsage::COPY_SRC,
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.,
            lod_max_clamp: 100.,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self { texture, view, sampler }
    }

    pub fn create_empty(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, format: wgpu::TextureFormat, label: &str) -> Self{
        let size = wgpu::Extent3d{
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let desc = wgpu::TextureDescriptor{
            label: Some(label),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                |  wgpu::TextureUsage::SAMPLED
                |  wgpu::TextureUsage::COPY_DST,
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.,
            lod_max_clamp: 100.,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self { texture, view, sampler }
    }
}
