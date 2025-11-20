use std::{fs, path::PathBuf, process::Command};

fn main() {
  // trigger recompilation when a new migration is added
  println!("cargo:rerun-if-changed=migrations");
  println!("cargo:rerun-if-changed=queries/*.sql");
  println!("cargo:rerun-if-changed=assets/");

  // Command::new("sqlc")
  //   .args(["generate"])
  //   .status()
  //   .expect("failed to run sqlc");

  std::fs::remove_dir_all("target/frontend").unwrap_or_default();

  Command::new("bunx")
    .args([
      "-y",
      "@tailwindcss/cli",
      "--input",
      "assets/css/main.css",
      "--output",
      "target/frontend/css/main.css",
      "--minify",
    ])
    .status()
    .expect("failed to build tailwind");

  Command::new("bun")
    .args([
      "build",
      "--minify",
      "--outdir=target/frontend/js",
      "--entry-naming",
      "[name].[hash].[ext]",
      "--asset-naming",
      "[name].[hash].[ext]",
      "assets/js/main.ts",
    ])
    .status()
    .expect("failed to build javascript");

  // std::fs::remove_file("target/frontend/css/main.css").unwrap_or_default();
  copy_files("public", "target/frontend");
}

fn copy_files<S: Into<PathBuf>, D: Into<PathBuf>>(src: S, dst: D) {
  let dir: PathBuf = src.into();
  let dest: PathBuf = dst.into();
  fs::create_dir_all(dest.as_path()).unwrap();
  let cloned_dir = dir.clone();
  let dir_name = cloned_dir.to_string_lossy();
  for entry in std::fs::read_dir(dir).unwrap_or_else(|_| panic!("failed to read dir {dir_name}")) {
    let entry = entry.expect("failed to read entry");
    if entry.file_type().unwrap().is_dir() {
      copy_files(entry.path().to_str().unwrap(), dest.join(entry.file_name()));
    } else {
      std::fs::copy(entry.path(), dest.join(entry.file_name())).expect("failed to copy file");
    }
  }
}
