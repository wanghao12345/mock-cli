use anyhow::Result;
use openapiv3::OpenAPI;



/// Generate a mock server from the OpenAPI spec.
pub fn generate_mock(spec: &OpenAPI) -> Result<()> {
    for (path, item) in spec.paths.iter() {
        println!("-----------------");
        println!("{:?}", path);
        println!("{:?}", item);
        println!("-----------------");
    }



    Ok(())
}