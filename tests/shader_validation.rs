const TRIANGLE_SHADER: &str = include_str!("../shaders/triangle.wgsl");

#[test]
fn triangle_shader_is_valid_wgsl_with_expected_entry_points() {
    let module = match naga::front::wgsl::parse_str(TRIANGLE_SHADER) {
        Ok(module) => module,
        Err(error) => panic!("triangle WGSL failed to parse: {error}"),
    };

    if let Err(error) = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    {
        panic!("triangle WGSL failed semantic validation: {error}");
    }

    assert!(
        module
            .entry_points
            .iter()
            .any(|entry| { entry.name == "vs_main" && entry.stage == naga::ShaderStage::Vertex })
    );
    assert!(
        module
            .entry_points
            .iter()
            .any(|entry| { entry.name == "fs_main" && entry.stage == naga::ShaderStage::Fragment })
    );
}
