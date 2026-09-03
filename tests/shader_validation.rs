const TRIANGLE_SHADER: &str = include_str!("../shaders/triangle.wgsl");
const VERTEX_BUFFER_SHADER: &str = include_str!("../shaders/vertex-buffer.wgsl");

fn parse_and_validate(label: &str, source: &str) -> naga::Module {
    let module = match naga::front::wgsl::parse_str(source) {
        Ok(module) => module,
        Err(error) => panic!("{label} WGSL failed to parse: {error}"),
    };

    if let Err(error) = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    {
        panic!("{label} WGSL failed semantic validation: {error}");
    }

    module
}

fn assert_entry_point(module: &naga::Module, name: &str, stage: naga::ShaderStage) {
    assert!(
        module
            .entry_points
            .iter()
            .any(|entry| entry.name == name && entry.stage == stage),
        "expected {stage:?} entry point {name}"
    );
}

#[test]
fn triangle_shader_is_valid_wgsl_with_expected_entry_points() {
    let module = parse_and_validate("triangle", TRIANGLE_SHADER);

    assert_entry_point(&module, "vs_main", naga::ShaderStage::Vertex);
    assert_entry_point(&module, "fs_main", naga::ShaderStage::Fragment);
}

#[test]
fn vertex_buffer_shader_is_valid_wgsl_with_expected_entry_points() {
    let module = parse_and_validate("vertex-buffer", VERTEX_BUFFER_SHADER);

    assert_entry_point(&module, "vs_main", naga::ShaderStage::Vertex);
    assert_entry_point(&module, "fs_main", naga::ShaderStage::Fragment);
}
