use std::process::Command;

fn main() {
    // Watch all workspace Cargo.toml files for dependency changes
    println!("cargo:rerun-if-changed=../Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-assets/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-backgrounds/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-camera/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-core/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-debug/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-mobs/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-particles/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-physics/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-player/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-projectiles/Cargo.toml");
    println!("cargo:rerun-if-changed=../thetawave-starter/Cargo.toml");

    // Generate dependency graph
    let output = Command::new("cargo")
        .args(["depgraph", "--workspace-only"])
        .output();

    match output {
        Ok(depgraph_output) => {
            if depgraph_output.status.success() {
                // Pipe to dot to generate PNG
                let mut dot_cmd = Command::new("dot")
                    .args(["-Tpng", "-o", "../dependency-graph.png"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .expect("Failed to start dot command");

                if let Some(stdin) = dot_cmd.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin
                        .write_all(&depgraph_output.stdout)
                        .expect("Failed to write to dot stdin");
                }

                let dot_result = dot_cmd.wait().expect("Failed to wait for dot command");
                if dot_result.success() {
                    println!("Successfully generated dependency-graph.png");
                } else {
                    eprintln!("Warning: dot command failed to generate PNG");
                }
            } else {
                eprintln!("Warning: cargo depgraph command failed");
                eprintln!("Make sure cargo-depgraph is installed: cargo install cargo-depgraph");
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to run cargo depgraph: {}", e);
            eprintln!("Make sure cargo-depgraph is installed: cargo install cargo-depgraph");
        }
    }
}
