use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use crate::scripting::{Capability, Command, Event, Module, ModuleId};

pub struct GraphicsModule {
    shader_cache: HashMap<String, ShaderInfo>,
    pipeline_configs: HashMap<String, PipelineConfig>,
}

#[derive(Debug)]
struct ShaderInfo {
    name: String,
    shader_type: ShaderType,
    source: String,
    compiled: bool,
}

#[derive(Debug)]
enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug)]
struct PipelineConfig {
    name: String,
    vertex_shader: String,
    fragment_shader: String,
    vertex_layout: Vec<VertexAttribute>,
    blend_state: BlendState,
}

#[derive(Debug)]
struct VertexAttribute {
    name: String,
    format: String,
    location: u32,
}

#[derive(Debug)]
enum BlendState {
    Opaque,
    AlphaBlend,
    Additive,
}

impl GraphicsModule {
    pub fn new() -> Self {
        Self {
            shader_cache: HashMap::new(),
            pipeline_configs: HashMap::new(),
        }
    }

    fn create_shader(&mut self, name: &str, shader_type: ShaderType, source: &str) -> Result<()> {
        self.shader_cache.insert(
            name.to_string(),
            ShaderInfo {
                name: name.to_string(),
                shader_type,
                source: source.to_string(),
                compiled: false,
            },
        );
        Ok(())
    }

    fn compile_shader(&mut self, name: &str) -> Result<()> {
        if let Some(shader) = self.shader_cache.get_mut(name) {
            // TODO: Implement actual shader compilation
            shader.compiled = true;
        }
        Ok(())
    }

    fn create_pipeline(&mut self, config: PipelineConfig) -> Result<()> {
        // Validate shaders exist
        if !self.shader_cache.contains_key(&config.vertex_shader)
            || !self.shader_cache.contains_key(&config.fragment_shader)
        {
            anyhow::bail!("Missing required shaders for pipeline");
        }

        self.pipeline_configs.insert(config.name.clone(), config);
        Ok(())
    }
}

impl Module for GraphicsModule {
    fn id(&self) -> ModuleId {
        ModuleId("graphics".to_string())
    }

    fn name(&self) -> &str {
        "Graphics"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "shader_management".to_string(),
                description: "Manages shader compilation and caching".to_string(),
                commands: vec![
                    Command {
                        name: "create_vertex_shader".to_string(),
                        description: "Creates a new vertex shader".to_string(),
                        args_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "source": { "type": "string" }
                            },
                            "required": ["name", "source"]
                        }),
                        return_schema: serde_json::json!({"type": "null"}),
                    },
                    Command {
                        name: "create_fragment_shader".to_string(),
                        description: "Creates a new fragment shader".to_string(),
                        args_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "source": { "type": "string" }
                            },
                            "required": ["name", "source"]
                        }),
                        return_schema: serde_json::json!({"type": "null"}),
                    },
                ],
                events: vec!["shader_compiled".to_string()],
            },
            Capability {
                name: "pipeline_management".to_string(),
                description: "Manages graphics pipeline configuration".to_string(),
                commands: vec![Command {
                    name: "create_pipeline".to_string(),
                    description: "Creates a new graphics pipeline".to_string(),
                    args_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "vertex_shader": { "type": "string" },
                            "fragment_shader": { "type": "string" },
                            "blend_state": { "type": "string" }
                        },
                        "required": ["name", "vertex_shader", "fragment_shader"]
                    }),
                    return_schema: serde_json::json!({"type": "null"}),
                }],
                events: vec!["pipeline_created".to_string()],
            },
        ]
    }

    fn handle_event(&self, _event: Event) -> Result<()> {
        Ok(())
    }

    fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        match command {
            "create_vertex_shader" => {
                let name = args["name"].as_str().unwrap();
                let source = args["source"].as_str().unwrap();
                self.create_shader(name, ShaderType::Vertex, source)?;
                Ok(Value::Null)
            }
            "create_fragment_shader" => {
                let name = args["name"].as_str().unwrap();
                let source = args["source"].as_str().unwrap();
                self.create_shader(name, ShaderType::Fragment, source)?;
                Ok(Value::Null)
            }
            "create_pipeline" => {
                let config = PipelineConfig {
                    name: args["name"].as_str().unwrap().to_string(),
                    vertex_shader: args["vertex_shader"].as_str().unwrap().to_string(),
                    fragment_shader: args["fragment_shader"].as_str().unwrap().to_string(),
                    vertex_layout: vec![], // TODO: Parse from args
                    blend_state: match args.get("blend_state").and_then(|v| v.as_str()) {
                        Some("alpha") => BlendState::AlphaBlend,
                        Some("additive") => BlendState::Additive,
                        _ => BlendState::Opaque,
                    },
                };
                self.create_pipeline(config)?;
                Ok(Value::Null)
            }
            _ => anyhow::bail!("Unknown command: {}", command),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_creation() {
        let mut module = GraphicsModule::new();

        let args = serde_json::json!({
            "name": "test_vertex",
            "source": "void main() {}"
        });

        assert!(module.execute_command("create_vertex_shader", args).is_ok());
    }

    #[test]
    fn test_pipeline_creation() {
        let mut module = GraphicsModule::new();

        // Create required shaders first
        let vs_args = serde_json::json!({
            "name": "test_vertex",
            "source": "void main() {}"
        });
        let fs_args = serde_json::json!({
            "name": "test_fragment",
            "source": "void main() {}"
        });
        module
            .execute_command("create_vertex_shader", vs_args)
            .unwrap();
        module
            .execute_command("create_fragment_shader", fs_args)
            .unwrap();

        // Create pipeline
        let pipeline_args = serde_json::json!({
            "name": "test_pipeline",
            "vertex_shader": "test_vertex",
            "fragment_shader": "test_fragment",
            "blend_state": "alpha"
        });

        assert!(module
            .execute_command("create_pipeline", pipeline_args)
            .is_ok());
    }
}
