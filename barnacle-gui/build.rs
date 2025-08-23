use slint_build::{CompilerConfiguration, compile_with_config};

fn main() {
    let config = CompilerConfiguration::new().with_style("cosmic".into());
    compile_with_config("ui/main.slint", config).expect("Slint build failed");
}
