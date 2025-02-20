use ash::vk;
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub element_id: u32,
}

impl TextVertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<TextVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        let position_offset = memoffset::offset_of!(TextVertex, position) as u32;
        let tex_coords_offset = memoffset::offset_of!(TextVertex, tex_coords) as u32;
        let color_offset = memoffset::offset_of!(TextVertex, color) as u32;
        let element_id_offset = memoffset::offset_of!(TextVertex, element_id) as u32;

        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: position_offset,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: tex_coords_offset,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: color_offset,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32_UINT,
                offset: element_id_offset,
            },
        ]
    }
}
