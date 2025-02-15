use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use crate::scripting::{Capability, Command, Event, Module, ModuleId};

pub struct PhysicsModule {
    bodies: HashMap<String, RigidBodyConfig>,
    colliders: HashMap<String, ColliderConfig>,
    constraints: HashMap<String, ConstraintConfig>,
}

#[derive(Debug)]
struct RigidBodyConfig {
    name: String,
    mass: f32,
    position: [f32; 3],
    rotation: [f32; 4],
    body_type: BodyType,
}

#[derive(Debug)]
enum BodyType {
    Dynamic,
    Static,
    Kinematic,
}

#[derive(Debug)]
struct ColliderConfig {
    name: String,
    shape: ColliderShape,
    body: String,
    friction: f32,
    restitution: f32,
}

#[derive(Debug)]
enum ColliderShape {
    Box { half_extents: [f32; 3] },
    Sphere { radius: f32 },
    Capsule { height: f32, radius: f32 },
}

#[derive(Debug)]
struct ConstraintConfig {
    name: String,
    constraint_type: ConstraintType,
    body_a: String,
    body_b: String,
}

#[derive(Debug)]
enum ConstraintType {
    Fixed,
    Hinge { axis: [f32; 3] },
    Distance { min: f32, max: f32 },
}

impl PhysicsModule {
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            constraints: HashMap::new(),
        }
    }

    fn create_rigid_body(&mut self, config: RigidBodyConfig) -> Result<()> {
        self.bodies.insert(config.name.clone(), config);
        Ok(())
    }

    fn create_collider(&mut self, config: ColliderConfig) -> Result<()> {
        // Validate body exists
        if !self.bodies.contains_key(&config.body) {
            anyhow::bail!("Referenced body does not exist: {}", config.body);
        }
        self.colliders.insert(config.name.clone(), config);
        Ok(())
    }

    fn create_constraint(&mut self, config: ConstraintConfig) -> Result<()> {
        // Validate bodies exist
        if !self.bodies.contains_key(&config.body_a) {
            anyhow::bail!("Referenced body A does not exist: {}", config.body_a);
        }
        if !self.bodies.contains_key(&config.body_b) {
            anyhow::bail!("Referenced body B does not exist: {}", config.body_b);
        }
        self.constraints.insert(config.name.clone(), config);
        Ok(())
    }
}

impl Module for PhysicsModule {
    fn id(&self) -> ModuleId {
        ModuleId("physics".to_string())
    }

    fn name(&self) -> &str {
        "Physics"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "rigid_body_management".to_string(),
                description: "Manages physics rigid bodies".to_string(),
                commands: vec![Command {
                    name: "create_rigid_body".to_string(),
                    description: "Creates a new rigid body".to_string(),
                    args_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "mass": { "type": "number" },
                            "position": {
                                "type": "array",
                                "items": { "type": "number" },
                                "minItems": 3,
                                "maxItems": 3
                            },
                            "body_type": { "type": "string", "enum": ["dynamic", "static", "kinematic"] }
                        },
                        "required": ["name", "mass", "position", "body_type"]
                    }),
                    return_schema: serde_json::json!({"type": "null"}),
                }],
                events: vec!["body_created".to_string()],
            },
            Capability {
                name: "collider_management".to_string(),
                description: "Manages physics colliders".to_string(),
                commands: vec![Command {
                    name: "create_collider".to_string(),
                    description: "Creates a new collider".to_string(),
                    args_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "body": { "type": "string" },
                            "shape": {
                                "type": "object",
                                "properties": {
                                    "type": { "type": "string" },
                                    "params": { "type": "object" }
                                }
                            },
                            "friction": { "type": "number" },
                            "restitution": { "type": "number" }
                        },
                        "required": ["name", "body", "shape"]
                    }),
                    return_schema: serde_json::json!({"type": "null"}),
                }],
                events: vec!["collider_created".to_string()],
            },
        ]
    }

    fn handle_event(&self, _event: Event) -> Result<()> {
        Ok(())
    }

    fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        match command {
            "create_rigid_body" => {
                let config = RigidBodyConfig {
                    name: args["name"].as_str().unwrap().to_string(),
                    mass: args["mass"].as_f64().unwrap() as f32,
                    position: [
                        args["position"][0].as_f64().unwrap() as f32,
                        args["position"][1].as_f64().unwrap() as f32,
                        args["position"][2].as_f64().unwrap() as f32,
                    ],
                    rotation: [1.0, 0.0, 0.0, 0.0], // Default quaternion
                    body_type: match args["body_type"].as_str().unwrap() {
                        "dynamic" => BodyType::Dynamic,
                        "static" => BodyType::Static,
                        "kinematic" => BodyType::Kinematic,
                        _ => anyhow::bail!("Invalid body type"),
                    },
                };
                self.create_rigid_body(config)?;
                Ok(Value::Null)
            }
            "create_collider" => {
                let shape = match args["shape"]["type"].as_str().unwrap() {
                    "box" => {
                        let params = &args["shape"]["params"];
                        ColliderShape::Box {
                            half_extents: [
                                params["half_extents"][0].as_f64().unwrap() as f32,
                                params["half_extents"][1].as_f64().unwrap() as f32,
                                params["half_extents"][2].as_f64().unwrap() as f32,
                            ],
                        }
                    }
                    "sphere" => ColliderShape::Sphere {
                        radius: args["shape"]["params"]["radius"].as_f64().unwrap() as f32,
                    },
                    _ => anyhow::bail!("Invalid collider shape"),
                };

                let config = ColliderConfig {
                    name: args["name"].as_str().unwrap().to_string(),
                    body: args["body"].as_str().unwrap().to_string(),
                    shape,
                    friction: args["friction"].as_f64().unwrap_or(0.5) as f32,
                    restitution: args["restitution"].as_f64().unwrap_or(0.0) as f32,
                };
                self.create_collider(config)?;
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
    fn test_rigid_body_creation() {
        let mut module = PhysicsModule::new();

        let args = serde_json::json!({
            "name": "test_body",
            "mass": 1.0,
            "position": [0.0, 1.0, 0.0],
            "body_type": "dynamic"
        });

        assert!(module.execute_command("create_rigid_body", args).is_ok());
    }

    #[test]
    fn test_collider_creation() {
        let mut module = PhysicsModule::new();

        // Create body first
        let body_args = serde_json::json!({
            "name": "test_body",
            "mass": 1.0,
            "position": [0.0, 1.0, 0.0],
            "body_type": "dynamic"
        });
        module
            .execute_command("create_rigid_body", body_args)
            .unwrap();

        // Create collider
        let collider_args = serde_json::json!({
            "name": "test_collider",
            "body": "test_body",
            "shape": {
                "type": "box",
                "params": {
                    "half_extents": [0.5, 0.5, 0.5]
                }
            },
            "friction": 0.5,
            "restitution": 0.1
        });

        assert!(module
            .execute_command("create_collider", collider_args)
            .is_ok());
    }
}
