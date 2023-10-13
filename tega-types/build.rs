use protobuf_codegen::Codegen;

fn main() {
	Codegen::new()
		.pure()
		.input("src/protos/tesla.proto")
		.include("src/protos")
		.cargo_out_dir("tesla-protos")
		.run_from_script();
}
