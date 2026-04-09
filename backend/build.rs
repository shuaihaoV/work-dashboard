use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=WORK_DASHBOARD_SKIP_FRONTEND_BUILD");
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/index.html");

    let frontend_dir = Path::new("../frontend");
    let dist_dir = frontend_dir.join("dist");
    ensure_placeholder_dist(&dist_dir);

    let skip = env::var("WORK_DASHBOARD_SKIP_FRONTEND_BUILD")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip {
        println!("cargo:warning=Skipping frontend build (WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true)");
        return;
    }

    if !frontend_dir.exists() {
        println!("cargo:warning=Frontend directory not found, using placeholder embedded assets");
        return;
    }

    let status = Command::new("pnpm")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status();

    match status {
        Ok(code) if code.success() => {
            println!("cargo:warning=Frontend build completed successfully");
        }
        Ok(code) => {
            println!(
                "cargo:warning=Frontend build exited with {code}; embedding existing dist directory"
            );
        }
        Err(err) => {
            println!(
                "cargo:warning=Failed to run frontend build command ({err}); embedding existing dist directory"
            );
        }
    }
}

fn ensure_placeholder_dist(dist_dir: &Path) {
    if let Err(err) = fs::create_dir_all(dist_dir) {
        println!("cargo:warning=Failed to create dist directory: {err}");
        return;
    }

    let index_path = dist_dir.join("index.html");
    if index_path.exists() {
        return;
    }

    let placeholder = r#"<!doctype html>
<html lang=\"en\">
  <head>
    <meta charset=\"UTF-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
    <title>Dashboard</title>
  </head>
  <body>
    <h1>Dashboard</h1>
    <p>Frontend assets are not built yet. Run: pnpm --dir ../frontend build</p>
  </body>
</html>
"#;

    if let Err(err) = fs::write(&index_path, placeholder) {
        println!("cargo:warning=Failed to write placeholder index.html: {err}");
    }
}
