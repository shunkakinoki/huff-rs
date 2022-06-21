#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

use huff_utils::{
    abi::*, artifact::*, ast::*, bytecode::*, error::CodegenError, prelude::CodegenErrorKind,
    types::EToken,
};
use std::fs;

/// ### Codegen
///
/// Code Generation Manager responsible for generating the code for the Huff Language.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Codegen<'a> {
    /// The Input AST
    pub ast: Option<Contract>,
    /// A cached codegen output artifact
    pub artifact: Option<Artifact>,
    /// Intermediate main bytecode store
    pub main_bytecode: Option<String>,
    /// Intermediate constructor bytecode store
    pub constructor_bytecode: Option<String>,

    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Codegen<'a> {
    /// Public associated function to instantiate a new Codegen instance.
    pub fn new() -> Self {
        Self {
            ast: None,
            artifact: None,
            main_bytecode: None,
            constructor_bytecode: None,
            phantom: std::marker::PhantomData,
        }
    }

    /// Generates main bytecode from a Contract AST
    ///
    /// # Arguments
    ///
    /// * `ast` - Optional Contract Abstract Syntax Tree
    pub fn roll(ast: Option<Contract>) -> Result<String, CodegenError<'a>> {
        let bytecode: String = String::default();

        // Grab the AST
        let _contract: &Contract = match &ast {
            Some(a) => a,
            None => {
                tracing::error!(
                    "Neither Codegen AST was set nor passed in as a parameter to Codegen::roll()!"
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingAst,
                    span: None,
                    token: None,
                });
            }
        };

        // TODO: main logic to create the main contract bytecode

        // Set bytecode and return
        // if self.main_bytecode.is_none() {
        //     self.main_bytecode = Some(bytecode.clone());
        // }
        Ok(bytecode)
    }

    /// Gracefully get the Contract AST
    pub fn graceful_ast_grab(&self, ast: Option<Contract>) -> Result<Contract, CodegenError> {
        match ast {
            Some(a) => Ok(a),
            None => match &self.ast {
                Some(a) => Ok(a.clone()),
                None => {
                    tracing::error!("Neither Codegen AST was set nor passed in as a parameter to Codegen::construct()!");
                    Err(CodegenError {
                        kind: CodegenErrorKind::MissingAst,
                        span: None,
                        token: None,
                    })
                }
            },
        }
    }

    /// Generates constructor bytecode from a Contract AST
    ///
    /// # Arguments
    ///
    /// * `ast` - Optional Contract Abstract Syntax Tree
    pub fn construct(ast: Option<Contract>) -> Result<String, CodegenError<'a>> {
        // Grab the AST
        let contract = match &ast {
            Some(a) => a,
            None => {
                tracing::error!(target: "codegen", "Neither Codegen AST was set nor passed in as a parameter to Codegen::construct()!");
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingAst,
                    span: None,
                    token: None,
                });
            }
        };

        // Find the constructor macro
        let c_macro: MacroDefinition = if let Some(m) = contract.find_macro_by_name("CONSTRUCTOR") {
            m
        } else {
            tracing::error!(target: "codegen", "'CONSTRUCTOR' Macro definition missing in AST!");
            return Err(CodegenError {
                kind: CodegenErrorKind::MissingConstructor,
                span: None,
                token: None,
            });
        };

        tracing::info!(target: "codegen", "CONSTRUCTOR MACRO FOUND: {:?}", c_macro);

        // For each MacroInvocation Statement, recurse into bytecode
        let recursed_bytecode: Vec<Byte> =
            Codegen::recurse_bytecode(c_macro.clone(), ast, &mut vec![c_macro])?;
        tracing::info!(target: "codegen", "RECURSED BYTECODE: {:?}", recursed_bytecode);

        let bytecode = recursed_bytecode.iter().map(|byte| byte.0.to_string()).collect();
        tracing::info!(target: "codegen", "FINAL BYTECODE: {:?}", bytecode);

        // Return
        Ok(bytecode)
    }

    /// Recurses a MacroDefinition to generate Bytecode
    pub fn recurse_bytecode(
        macro_def: MacroDefinition,
        ast: Option<Contract>,
        scope: &mut Vec<MacroDefinition>,
    ) -> Result<Vec<Byte>, CodegenError<'a>> {
        let mut final_bytes: Vec<Byte> = vec![];
        tracing::info!(target: "codegen", "RECURSING MACRO DEFINITION");

        // Grab the AST
        let contract = match &ast {
            Some(a) => a,
            None => {
                tracing::error!(target: "codegen", "Neither Codegen AST was set nor passed in as a parameter to Codegen::construct()!");
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingAst,
                    span: None,
                    token: None,
                });
            }
        };

        // Generate the macro bytecode
        let irb = macro_def.to_irbytecode()?;
        tracing::info!(target: "codegen", "GENERATED IRBYTECODE: {:?}", irb);
        let irbz = irb.0;

        for irbyte in irbz.iter() {
            match irbyte.clone() {
                IRByte::Byte(b) => final_bytes.push(b.clone()),
                IRByte::Constant(name) => {
                    let constant = if let Some(m) = contract
                        .constants
                        .iter()
                        .filter(|const_def| const_def.name == name)
                        .cloned()
                        .collect::<Vec<ConstantDefinition>>()
                        .get(0)
                    {
                        m.clone()
                    } else {
                        tracing::error!(target: "codegen", "MISSING CONTRACT MACRO \"{}\"", name);

                        // TODO we should try and find the constant defined in other files here
                        return Err(CodegenError {
                            kind: CodegenErrorKind::MissingConstantDefinition,
                            span: None,
                            token: None,
                        });
                    };

                    tracing::info!(target: "codegen", "FOUND CONSTANT DEFINITION: {:?}", constant);

                    let push_bytes = match constant.value {
                        ConstVal::Literal(l) => {
                            let hex_literal: String = hex::encode(l);
                            format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal)
                        }
                        ConstVal::FreeStoragePointer(_fsp) => {
                            // TODO: we need to grab the using the offset?
                            let offset: u8 = 0;
                            let hex_literal: String = hex::encode([offset]);
                            format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal)
                        }
                    };
                    tracing::info!(target: "codegen", "PUSH BYTES: {:?}", push_bytes);
                    final_bytes.push(Byte(push_bytes))
                }
                IRByte::Statement(s) => {
                    match s {
                        Statement::MacroInvocation(mi) => {
                            // Get the macro that matches this invocation and turn into bytecode
                            let ir_macro =
                                if let Some(m) = contract.find_macro_by_name(&mi.macro_name) {
                                    m
                                } else {
                                    // TODO: this is where the file imports must be resolved .. in
                                    // case macro definition is external
                                    tracing::warn!(
                                        "Invoked Macro \"{}\" not found in Contract",
                                        mi.macro_name
                                    );
                                    return Err(CodegenError {
                                        kind: CodegenErrorKind::MissingMacroDefinition,
                                        span: None,
                                        token: None,
                                    });
                                };

                            tracing::info!(target: "codegen", "FOUND INNER MACRO: {:?}", ir_macro);

                            // Recurse
                            scope.push(ir_macro.clone());
                            let recursed_bytecode: Vec<Byte> = if let Ok(bytes) =
                                Codegen::recurse_bytecode(ir_macro.clone(), ast.clone(), scope)
                            {
                                bytes
                            } else {
                                tracing::error!(
                                    "Codegen failed to recurse into macro {}",
                                    ir_macro.name
                                );
                                return Err(CodegenError {
                                    kind: CodegenErrorKind::FailedMacroRecursion,
                                    span: None,
                                    token: None,
                                });
                            };
                            final_bytes = final_bytes
                                .iter()
                                .cloned()
                                .chain(recursed_bytecode.iter().cloned())
                                .collect();
                        }
                        s => {
                            tracing::error!(target: "codegen", "UNEXPECTED STATEMENT: {:?}", s);
                            return Err(CodegenError {
                                kind: CodegenErrorKind::InvalidMacroStatement,
                                span: None,
                                token: None,
                            });
                        }
                    }
                }
                IRByte::ArgCall(arg_name) => {
                    // TODO: Check our scope, loop through all macros, all statements, to see if out
                    // arg is defined as a jumpdest match
                    tracing::info!(target: "codegen", "FOUND ARG CALL TO \"{}\"", arg_name);
                }
            }
        }

        Ok(final_bytes)
    }

    /// Generate a codegen artifact
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of Tokens representing constructor arguments
    /// * `main_bytecode` - The compiled MAIN Macro bytecode
    /// * `constructor_bytecode` - The compiled `CONSTRUCTOR` Macro bytecode
    pub fn churn(
        &mut self,
        args: Vec<ethers::abi::token::Token>,
        main_bytecode: &str,
        constructor_bytecode: &str,
    ) -> Result<Artifact, CodegenError> {
        let mut artifact: &mut Artifact = if let Some(art) = &mut self.artifact {
            art
        } else {
            self.artifact = Some(Artifact::default());
            self.artifact.as_mut().unwrap()
        };

        let contract_length = main_bytecode.len() / 2;
        let constructor_length = constructor_bytecode.len() / 2;

        let contract_size = format!("{:04x}", contract_length);
        let contract_code_offset = format!("{:04x}", 13 + constructor_length);

        let encoded: Vec<Vec<u8>> =
            args.iter().map(|tok| ethers::abi::encode(&[tok.clone()])).collect();
        let hex_args: Vec<String> = encoded.iter().map(|tok| hex::encode(tok.as_slice())).collect();
        let constructor_args = hex_args.join("");

        // Generate the final bytecode
        let bootstrap_code = format!("61{}8061{}6000396000f3", contract_size, contract_code_offset);
        let constructor_code = format!("{}{}", constructor_bytecode, bootstrap_code);
        artifact.bytecode = format!("{}{}{}", constructor_code, main_bytecode, constructor_args);
        artifact.runtime = main_bytecode.to_string();
        Ok(artifact.clone())
    }

    /// Encode constructor arguments as ethers::abi::token::Token
    pub fn encode_constructor_args(args: Vec<String>) -> Vec<ethers::abi::token::Token> {
        let tokens: Vec<ethers::abi::token::Token> =
            args.iter().map(|tok| EToken::try_from(tok.clone()).unwrap().0).collect();
        tokens
    }

    /// Export
    ///
    /// Writes a Codegen Artifact out to the specified file.
    ///
    /// # Arguments
    ///
    /// * `out` - Output location to write the serialized json artifact to.
    pub fn export(&self, output: String) -> Result<(), CodegenError> {
        if let Some(art) = &self.artifact {
            let serialized_artifact = serde_json::to_string(art).unwrap();
            fs::write(output, serialized_artifact).expect("Unable to write file");
        } else {
            tracing::warn!(
                target: "codegen",
                "Failed to export the compile artifact to the specified output location {}!",
                output
            );
        }
        Ok(())
    }

    /// Abi Generation
    ///
    /// Generates an ABI for the given Ast.
    /// Stores the generated ABI in the Codegen `artifact`.
    ///
    /// # Arguments
    ///
    /// * `ast` - The Contract Abstract Syntax Tree
    /// * `output` - An optional output path
    pub fn abigen(&mut self, ast: Contract, output: Option<String>) -> Result<Abi, CodegenError> {
        let abi: Abi = ast.into();

        // Set the abi on self
        match &mut self.artifact {
            Some(artifact) => {
                artifact.abi = Some(abi.clone());
            }
            None => {
                self.artifact = Some(Artifact { abi: Some(abi.clone()), ..Default::default() });
            }
        }

        // If an output's specified, write the artifact out
        if let Some(o) = output {
            if self.export(o).is_err() {
                // !! We should never get here since we set the artifact above !! //
            }
        }

        // Return the abi
        Ok(abi)
    }
}
