fn main() {
    table_gen::generate_tables();
    println!("cargo:rerun-if-changed=../table-gen/src");
}
