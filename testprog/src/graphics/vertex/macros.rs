#[macro_export]
macro_rules! define_vertex {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type
            ),*
        }

        unsafe impl $crate::graphics::vertex::VertexAttributes for $name {
            fn get_attributes() -> std::collections::HashMap<String, vulkano::pipeline::graphics::vertex_input::VertexMemberInfo> {
                let mut attrs = std::collections::HashMap::new();
                let mut offset = 0;

                $(
                    attrs.insert(
                        stringify!($field_name).to_string(),
                        vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                            format: <$field_type as $crate::graphics::vertex::FormatFor>::FORMAT,
                            offset,
                            num_elements: 1,
                        }
                    );
                    offset += std::mem::size_of::<$field_type>() as u32;
                )*

                attrs
            }

            fn get_stride() -> u32 {
                std::mem::size_of::<Self>() as u32
            }

            fn get_input_rate() -> vulkano::pipeline::graphics::vertex_input::VertexInputRate {
                vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex
            }
        }

        unsafe impl vulkano::pipeline::graphics::vertex_input::Vertex for $name {
            fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                    stride: Self::get_stride(),
                    input_rate: Self::get_input_rate(),
                    members: Self::get_attributes(),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_instance_data {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type
            ),*
        }

        unsafe impl $crate::graphics::vertex::VertexAttributes for $name {
            fn get_attributes() -> std::collections::HashMap<String, vulkano::pipeline::graphics::vertex_input::VertexMemberInfo> {
                let mut attrs = std::collections::HashMap::new();
                let mut offset = 0;

                $(
                    attrs.insert(
                        stringify!($field_name).to_string(),
                        vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                            format: <$field_type as $crate::graphics::vertex::FormatFor>::FORMAT,
                            offset,
                            num_elements: 1,
                        }
                    );
                    offset += std::mem::size_of::<$field_type>() as u32;
                )*

                attrs
            }

            fn get_stride() -> u32 {
                std::mem::size_of::<Self>() as u32
            }

            fn get_input_rate() -> vulkano::pipeline::graphics::vertex_input::VertexInputRate {
                vulkano::pipeline::graphics::vertex_input::VertexInputRate::Instance { divisor: 1 }
            }
        }

        unsafe impl vulkano::pipeline::graphics::vertex_input::Vertex for $name {
            fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                    stride: Self::get_stride(),
                    input_rate: Self::get_input_rate(),
                    members: Self::get_attributes(),
                }
            }
        }
    };
}

/// Trait for mapping Rust types to Vulkan formats
pub trait FormatFor {
    const FORMAT: vulkano::format::Format;
}

// Implement FormatFor for common types
impl FormatFor for [f32; 2] {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32_SFLOAT;
}

impl FormatFor for [f32; 3] {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32B32_SFLOAT;
}

impl FormatFor for [f32; 4] {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32B32A32_SFLOAT;
}

impl FormatFor for glam::Vec2 {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32_SFLOAT;
}

impl FormatFor for glam::Vec3 {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32B32_SFLOAT;
}

impl FormatFor for glam::Vec4 {
    const FORMAT: vulkano::format::Format = vulkano::format::Format::R32G32B32A32_SFLOAT;
}
