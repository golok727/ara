use ara::Context;
use pollster::FutureExt;

mod basic;
struct Example {
    name: &'static str,
    run: fn(gpu: ara::Context) -> (),
}

const EXAMPLES: &[Example] = &[
    Example {
        name: "basic",
        run: basic::run,
    },
];

fn main() {
    let example_name = std::env
        ::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| {
            println!("Usage: cargo run <example_name>");
            println!("Available examples:");
            for example in EXAMPLES {
                println!("  - {}", example.name);
            }
            std::process::exit(1);
        });

    let example = EXAMPLES.iter()
        .find(|example| example.name == example_name)
        .unwrap_or_else(|| {
            println!("Example '{}' not found", example_name);
            std::process::exit(1);
        });

    let gpu = Context::new(
        &(ara::gpu::ContextSpecification {
            power_preference: ara::gpu::PowerPreference::HighPerformance,
            backends: ara::gpu::Backends::all(),
            ..Default::default()
        })
    )
        .block_on()
        .expect("Failed to create GPU context");

    println!("Running example: {}", example.name);
    (example.run)(gpu);
}
