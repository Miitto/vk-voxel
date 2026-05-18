use shader_slang::Downcast;

mod kw {
    syn::custom_keyword!(debug);
    syn::custom_keyword!(none);
    syn::custom_keyword!(default);
    syn::custom_keyword!(high);
}

struct MacroInput {
    opt_level: shader_slang::OptimizationLevel,
    debug: shader_slang::DebugInfoLevel,
    base_path: String,
    files: Vec<String>,
}

struct OptLvl(pub shader_slang::OptimizationLevel);

impl syn::parse::Parse for OptLvl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::none) {
            input.parse::<kw::none>()?;
            Ok(OptLvl(shader_slang::OptimizationLevel::None))
        } else if input.peek(kw::default) {
            input.parse::<kw::default>()?;
            Ok(OptLvl(shader_slang::OptimizationLevel::Default))
        } else if input.peek(kw::high) {
            input.parse::<kw::high>()?;
            Ok(OptLvl(shader_slang::OptimizationLevel::High))
        } else {
            Ok(OptLvl(shader_slang::OptimizationLevel::Default))
        }
    }
}

impl syn::parse::Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut debug = shader_slang::DebugInfoLevel::None;

        let base_path = input.parse::<syn::LitStr>()?.value();
        input.parse::<syn::Token![,]>()?;

        if input.peek(kw::debug) {
            input.parse::<kw::debug>()?;
            debug = shader_slang::DebugInfoLevel::Maximal;
            input.parse::<syn::Token![,]>()?;
        }

        let opt_level = input.parse::<OptLvl>()?.0;
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }

        let lit_strs =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;

        let files = lit_strs.into_iter().map(|s| s.value()).collect();

        Ok(MacroInput {
            base_path,
            opt_level,
            debug,
            files,
        })
    }
}

#[proc_macro]
pub fn shaders(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as MacroInput);

    let session = create_session(&input);

    let mut shaders = Vec::with_capacity(input.files.len());
    for file in &input.files {
        match compile_file(file, &session) {
            Ok(shader_data) => shaders.push(shader_data),
            Err(e) => panic!("Error compiling shader {}: {}", file, e),
        }
    }

    let mut shader_tokens = Vec::with_capacity(shaders.len());
    for shader in shaders {
        let name = syn::Ident::new(&shader.name, proc_macro2::Span::call_site());
        let code = &shader.code;
        let entry_points = shader.entry_points.into_iter().map(|ep| {
            let ep_name = syn::Ident::new(&ep.name, proc_macro2::Span::call_site());
            let stage = match ep.stage {
                shader_slang::Stage::Vertex => {
                    quote::quote! { ::shader::ShaderStage::Vertex }
                }
                shader_slang::Stage::Fragment => {
                    quote::quote! { ::shader::ShaderStage::Fragment }
                }
                shader_slang::Stage::Compute => {
                    quote::quote! { ::shader::ShaderStage::Compute }
                }
                _ => unimplemented!("Unsupported shader stage"),
            };
            quote::quote! {
                ::shader::EntryPoint {
                    name: stringify!(#ep_name),
                    stage: #stage,
                }
            }
        });

        shader_tokens.push(quote::quote! {
            pub const #name: ::shader::Shader = ::shader::Shader {
                name: stringify!(#name),
                code: &[#(#code),*],
                entry_points: &[#(#entry_points),*],
            };
        });
    }

    let base_path = &input.base_path;
    let files = input.files.join(",");

    quote::quote! {
        #(#shader_tokens)*
    }
    .into()
}

struct Session {
    global: shader_slang::GlobalSession,
    session: shader_slang::Session,
    base_path: std::path::PathBuf,
}

impl std::ops::Deref for Session {
    type Target = shader_slang::Session;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

fn create_session(input: &MacroInput) -> Session {
    let global_session =
        shader_slang::GlobalSession::new().expect("Failed to create shader slang global session");

    let session_options = shader_slang::CompilerOptions::default()
        .optimization(input.opt_level)
        .debug_information(input.debug)
        .matrix_layout_row(true)
        .emit_spirv_directly(true)
        .vulkan_use_entry_point_name(true);

    let target_desc = shader_slang::TargetDesc::default()
        .format(shader_slang::CompileTarget::Spirv)
        .profile(global_session.find_profile("spirv_1_6"));

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let shader_dir = current_dir.join(input.base_path.clone());
    println!("Search path: {}", shader_dir.display());
    let c_str = std::ffi::CString::new(shader_dir.to_str().unwrap())
        .expect("Failed to convert shader directory to CString");

    let targets = [target_desc];
    let search_paths = [c_str.as_ptr()];

    let session_desc = shader_slang::SessionDesc::default()
        .options(&session_options)
        .targets(&targets)
        .search_paths(&search_paths);

    let session = global_session
        .create_session(&session_desc)
        .expect("Failed to create shader slang session");

    Session {
        global: global_session,
        session,
        base_path: shader_dir,
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),
    #[error("Failed to load module: {0}")]
    ModuleLoad(#[source] shader_slang::Error),
    #[error("No entry points found in shader")]
    NoEntryPoints,
}

struct EntryPointData {
    name: String,
    stage: shader_slang::Stage,
}

struct ShaderData {
    name: String,
    code: Vec<u8>,
    entry_points: Vec<EntryPointData>,
}

fn compile_file(file: &str, session: &Session) -> Result<ShaderData, Error> {
    println!("Compiling shader: {}", file);

    let full_path = session.base_path.join(file);
    if !full_path.exists() {
        return Err(Error::FileNotFound(full_path));
    }

    let module = session.load_module(file).map_err(Error::ModuleLoad)?;
    println!("Loaded module: {}", file);

    if module.entry_point_count() == 0 {
        return Err(Error::NoEntryPoints);
    }

    let entry_points = module.entry_points();
    let mods = std::iter::once(module.downcast().clone())
        .chain(entry_points.map(|ep| ep.downcast().clone()))
        .collect::<Vec<_>>();

    println!("Creating composite component type for module: {}", file);
    let program = session
        .create_composite_component_type(mods.as_slice())
        .expect("Failed to create composite component type");

    println!("Linking shader program for module: {}", file);
    let linked_program = program.link().expect("Failed to link shader program");

    let name = file.strip_suffix(".slang").unwrap_or(file).to_string();

    println!("Getting target code for module: {}", file);
    let code = linked_program
        .target_code(0)
        .expect("Failed to get target code")
        .as_slice()
        .to_vec();

    let mut entry_points = Vec::new();
    println!("Reflecting entry points for module: {}", file);
    let reflection = linked_program
        .layout(0)
        .expect("Failed to get program layout");

    println!("Entry points for module {}:", file);
    reflection.entry_points().for_each(|ep| {
        println!("- {} ({:?})", ep.name(), ep.stage());
        entry_points.push(EntryPointData {
            name: ep.name().to_string(),
            stage: ep.stage(),
        });
    });

    Ok(ShaderData {
        name,
        code,
        entry_points,
    })
}
