use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use versiontracking::{
    scripting::{Interpreter, ScriptConfig},
    Analyzer, CodeGenerator, GenerationTarget, GeneratorConfig,
};

fn analyze_benchmark(c: &mut Criterion) {
    let source = r#"
        pub struct Test {
            field1: String,
            field2: Vec<u32>,
        }

        impl Test {
            pub fn new(field1: String) -> Self {
                Self {
                    field1,
                    field2: Vec::new(),
                }
            }

            pub fn add_number(&mut self, num: u32) {
                self.field2.push(num);
            }

            pub fn process(&self) -> Result<(), std::io::Error> {
                std::fs::write("test.txt", &self.field1)?;
                std::thread::sleep(std::time::Duration::from_secs(1));
                Ok(())
            }
        }
    "#;

    let mut group = c.benchmark_group("analysis");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("pattern_detection", |b| {
        let analyzer = Analyzer::new();
        b.iter(|| analyzer.analyze(black_box(&syn::parse_str(source).unwrap())));
    });

    group.bench_function("breaking_changes", |b| {
        let analyzer = Analyzer::new();
        let old_code = syn::parse_str(source).unwrap();
        let new_code = syn::parse_str(
            r#"
            pub struct Test {
                field1: String,
                field2: Vec<u32>,
                field3: Option<String>, // Added field
            }
            "#,
        )
        .unwrap();

        b.iter(|| analyzer.compare_versions(black_box(&old_code), black_box(&new_code)));
    });

    group.finish();
}

fn generation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("struct_generation", |b| {
        let generator = CodeGenerator::new(GeneratorConfig::default());
        let target = GenerationTarget::Struct {
            name: "TestStruct".to_string(),
            fields: vec![
                "field1: String".to_string(),
                "field2: Vec<u32>".to_string(),
                "field3: Option<String>".to_string(),
            ],
        };

        b.iter(|| async { generator.generate(black_box(target.clone())).await.unwrap() });
    });

    group.bench_function("async_conversion", |b| {
        let generator = CodeGenerator::new(GeneratorConfig {
            transform_config: Default::default(),
            documentation: true,
            custom_templates: false,
        });

        let target = GenerationTarget::Function {
            name: "test_function".to_string(),
            is_async: true,
            is_unsafe: false,
        };

        b.iter(|| async { generator.generate(black_box(target.clone())).await.unwrap() });
    });

    group.finish();
}

fn script_execution_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    let script = r#"
        analyze src/lib.rs {
            detect blocking_io
            detect unsafe_blocks
            check_safety = true
        }

        generate TestStruct struct {
            field name: String
            field value: u32
            async = "true"
            validation = "true"
        }

        apply_pattern builder TestStruct {
            validation = "true"
            doc = "true"
        }
    "#;

    group.bench_function("script_parsing", |b| {
        b.iter(|| versiontracking::scripting::parser::parse_script(black_box(script)));
    });

    group.bench_function("script_execution", |b| {
        let mut interpreter = Interpreter::new();
        b.iter(|| async { interpreter.execute_script(black_box(script)).await.unwrap() });
    });

    group.finish();
}

fn large_codebase_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_codebase");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);

    // Generate large source code
    let mut source = String::with_capacity(100_000);
    for i in 0..1000 {
        source.push_str(&format!(
            r#"
            pub struct Struct{0} {{
                field1: String,
                field2: Vec<u32>,
            }}

            impl Struct{0} {{
                pub fn new() -> Self {{
                    Self {{
                        field1: String::new(),
                        field2: Vec::new(),
                    }}
                }}

                pub fn process(&mut self) {{
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }}
            }}
            "#,
            i
        ));
    }

    group.bench_function("analyze_large_codebase", |b| {
        let analyzer = Analyzer::new();
        b.iter(|| analyzer.analyze(black_box(&syn::parse_str(&source).unwrap())));
    });

    group.bench_function("generate_large_codebase", |b| {
        let generator = CodeGenerator::new(GeneratorConfig::default());
        let targets: Vec<_> = (0..100)
            .map(|i| GenerationTarget::Struct {
                name: format!("Generated{}", i),
                fields: vec!["field1: String".to_string(), "field2: Vec<u32>".to_string()],
            })
            .collect();

        b.iter(|| async {
            for target in targets.iter() {
                generator.generate(black_box(target.clone())).await.unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    analyze_benchmark,
    generation_benchmark,
    script_execution_benchmark,
    large_codebase_benchmark
);
criterion_main!(benches);
