// src/resources.rs
use crate::parts::texture;

pub async fn load_binary(file_name: &str) -> Vec<u8> {
    let path = std::path::Path::new("assets").join(file_name);
    std::fs::read(path).expect("读取文件失败")
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> texture::Texture {
    let data = load_binary(file_name).await;
    texture::Texture::from_bytes(device, queue, &data, file_name).unwrap()
}
