const TRIANGLE_SHADER: &str = include_str!("../shaders/triangle.wgsl");

#[test]
fn triangle_shader_is_valid_wgsl_with_expected_entry_points() {
    let module = match naga::front::wgsl::parse_str(TRIANGLE_SHADER) {
        Ok(module) => module,
        Err(error) => panic!("triangle WGSL failed to parse: {error}"),
    };

    assert!(
        module
            .entry_points
            .iter()
            .any(|entry| entry.name == "vs_main")
    );
    assert!(
        module
            .entry_points
            .iter()
            .any(|entry| entry.name == "fs_main")
    );
}
