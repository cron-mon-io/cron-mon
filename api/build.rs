fn main() {
    println!("cargo:rerun-if-changed=src/infrastructure/migrations");
}
