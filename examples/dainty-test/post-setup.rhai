// Post-generation setup script
// Trust mise configuration and install dependencies

print("🔧 Setting up mise configuration...");

// Trust the mise configuration
system::command("mise", ["trust"]);

print("📦 Installing dependencies with mise...");

// Install mise dependencies
system::command("mise", ["install"]);

print("🔧 Adding Rust components...");

system::command("rustup", ["component", "add", "llvm-tools", "rust-analyzer", "rustfmt", "clippy"]);

print("✅ Setup complete! Run 'mise dev' to start the development server.");
