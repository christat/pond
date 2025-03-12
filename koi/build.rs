use glob::glob;
use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

// TODO for glsl shaders (e.g.):
// glslangValidator -V -e main_vs --source-entrypoint main .\vertex.vert -o .\vertex.spv

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in glob("../shaders/*")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|path| !path.ends_with("glsl"))
    {
        SpirvBuilder::new(path.as_path().as_os_str(), "spirv-unknown-spv1.5")
            .capability(Capability::ImageQuery)
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    }
    Ok(())
}
