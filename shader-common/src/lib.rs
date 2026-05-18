pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

pub struct EntryPoint {
    pub name: &'static str,
    pub stage: ShaderStage,
}

pub struct Shader {
    pub name: &'static str,
    pub code: &'static [u8],
    pub entry_points: &'static [EntryPoint],
}
