use glob::glob;
use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in glob("../shaders/*").unwrap().filter_map(Result::ok) {
        SpirvBuilder::new(path.as_path().as_os_str(), "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    }
    Ok(())
}
