use jtd_codegen::target::{self, inflect, metadata};
use jtd_codegen::Result;
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;

lazy_static! {
    static ref KEYWORDS: BTreeSet<String> = include_str!("keywords")
        .lines()
        .map(str::to_owned)
        .collect();
    static ref INITIALISMS: BTreeSet<String> = include_str!("initialisms")
        .lines()
        .map(str::to_owned)
        .collect();
    static ref PACKAGE_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(inflect::KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::snake_case()),
        ));
    static ref ITEM_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(inflect::KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::CombiningInflector::new(inflect::Case::pascal_case_with_initialisms(
                INITIALISMS.clone()
            ))
        ));
    static ref FIELD_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(inflect::KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::pascal_case_with_initialisms(
                INITIALISMS.clone()
            ))
        ));
}

pub struct Target {
    package: String,
}

impl Target {
    pub fn new(package: String) -> Self {
        Self { package }
    }
}

impl jtd_codegen::target::Target for Target {
    type FileState = FileState;

    fn strategy(&self) -> target::Strategy {
        target::Strategy {
            file_partitioning: target::FilePartitioningStrategy::SingleFile(format!(
                "{}.go",
                PACKAGE_NAMING_CONVENTION.inflect(&[self.package.clone()]),
            )),
            enum_member_naming: target::EnumMemberNamingStrategy::Unmodularized,
            optional_property_handling: target::OptionalPropertyHandlingStrategy::WrapWithNullable,
            booleans_are_nullable: false,
            int8s_are_nullable: false,
            uint8s_are_nullable: false,
            int16s_are_nullable: false,
            uint16s_are_nullable: false,
            int32s_are_nullable: false,
            uint32s_are_nullable: false,
            float32s_are_nullable: false,
            float64s_are_nullable: false,
            strings_are_nullable: false,
            timestamps_are_nullable: false,
            arrays_are_nullable: true,
            dicts_are_nullable: true,
            aliases_are_nullable: false,
            enums_are_nullable: false,
            structs_are_nullable: false,
            discriminators_are_nullable: false,
        }
    }

    fn name(&self, kind: target::NameableKind, parts: &[String]) -> String {
        match kind {
            target::NameableKind::Type => ITEM_NAMING_CONVENTION.inflect(parts),
            target::NameableKind::Field => FIELD_NAMING_CONVENTION.inflect(parts),
            target::NameableKind::EnumMember => ITEM_NAMING_CONVENTION.inflect(parts),
        }
    }

    fn expr(
        &self,
        state: &mut FileState,
        metadata: metadata::Metadata,
        expr: target::Expr,
    ) -> String {
        if let Some(s) = metadata.get("goType").and_then(|v| v.as_str()) {
            return s.into();
        }

        match expr {
            target::Expr::Empty => "interface{}".into(),
            target::Expr::Boolean => "bool".into(),
            target::Expr::Int8 => "int8".into(),
            target::Expr::Uint8 => "uint8".into(),
            target::Expr::Int16 => "int16".into(),
            target::Expr::Uint16 => "uint16".into(),
            target::Expr::Int32 => "int32".into(),
            target::Expr::Uint32 => "uint32".into(),
            target::Expr::Float32 => "float32".into(),
            target::Expr::Float64 => "float64".into(),
            target::Expr::String => "string".into(),
            target::Expr::Timestamp => {
                state.imports.insert("time".into());
                "time.Time".into()
            }
            target::Expr::ArrayOf(sub_expr) => format!("[]{}", sub_expr),
            target::Expr::DictOf(sub_expr) => format!("map[string]{}", sub_expr),
            target::Expr::NullableOf(sub_expr, _) => format!("*{}", sub_expr),
        }
    }

    fn item(
        &self,
        out: &mut dyn Write,
        state: &mut FileState,
        item: target::Item,
    ) -> Result<Option<String>> {
        Ok(match item {
            target::Item::Auxiliary { .. } => {
                // No auxiliary files needed.
                None
            }

            target::Item::Preamble => {
                writeln!(
                    out,
                    "// Code generated by jtd-codegen for Go v{}. DO NOT EDIT.",
                    env!("CARGO_PKG_VERSION")
                )?;
                writeln!(out)?;

                writeln!(out, "package {}", self.package)?;

                if !state.imports.is_empty() {
                    writeln!(out)?;

                    let imports: Vec<String> = state.imports.iter().cloned().collect();
                    if imports.len() == 1 {
                        writeln!(out, "import {:?}", imports[0])?;
                    } else {
                        writeln!(out, "import (")?;
                        for import in imports {
                            writeln!(out, "\t{:?}", import)?;
                        }
                        writeln!(out, ")")?;
                    }
                }

                None
            }

            target::Item::Postamble => None,

            target::Item::Alias {
                metadata,
                name,
                type_,
            } => {
                writeln!(out)?;
                write!(out, "{}", description(&metadata, 0))?;
                writeln!(out, "type {} = {}", name, type_)?;

                None
            }

            target::Item::Enum {
                metadata,
                name,
                members,
            } => {
                if let Some(s) = metadata.get("goType").and_then(|v| v.as_str()) {
                    return Ok(Some(s.into()));
                }

                writeln!(out)?;
                write!(out, "{}", description(&metadata, 0))?;
                writeln!(out, "type {} string", name)?;

                writeln!(out)?;
                writeln!(out, "const (")?;
                for (index, member) in members.into_iter().enumerate() {
                    if index != 0 {
                        writeln!(out)?;
                    }

                    write!(
                        out,
                        "{}",
                        enum_variant_description(&metadata, 0, &member.json_value)
                    )?;
                    writeln!(out, "\t{} {} = {:?}", member.name, name, member.json_value)?;
                }
                writeln!(out, ")")?;

                None
            }

            target::Item::Struct {
                metadata,
                name,
                has_additional: _,
                fields,
            } => {
                if let Some(s) = metadata.get("goType").and_then(|v| v.as_str()) {
                    return Ok(Some(s.into()));
                }

                writeln!(out)?;
                write!(out, "{}", description(&metadata, 0))?;
                writeln!(out, "type {} struct {{", name)?;
                for (index, field) in fields.into_iter().enumerate() {
                    if index != 0 {
                        writeln!(out)?;
                    }

                    write!(out, "{}", description(&field.metadata, 1))?;

                    if field.optional {
                        writeln!(
                            out,
                            "\t{} {} `json:\"{},omitempty\"`",
                            field.name, field.type_, field.json_name
                        )?;
                    } else {
                        writeln!(
                            out,
                            "\t{} {} `json:\"{}\"`",
                            field.name, field.type_, field.json_name
                        )?;
                    }
                }
                writeln!(out, "}}")?;

                None
            }

            target::Item::Discriminator {
                metadata,
                name,
                tag_field_name,
                tag_json_name,
                variants,
            } => {
                if let Some(s) = metadata.get("goType").and_then(|v| v.as_str()) {
                    return Ok(Some(s.into()));
                }

                state.imports.insert("encoding/json".into());
                state.imports.insert("fmt".into());

                writeln!(out)?;
                write!(out, "{}", description(&metadata, 0))?;
                writeln!(out, "type {} struct {{", name)?;
                writeln!(out, "\t{} string", tag_field_name)?;
                for variant in &variants {
                    writeln!(out)?;
                    writeln!(out, "\t{} {}", &variant.field_name, &variant.type_name)?;
                }
                writeln!(out, "}}")?;

                writeln!(out)?;
                writeln!(out, "func (v {}) MarshalJSON() ([]byte, error) {{", name)?;
                writeln!(out, "\tswitch v.{} {{", tag_field_name)?;
                for variant in &variants {
                    writeln!(out, "\tcase {:?}:", variant.tag_value)?;
                    writeln!(out, "\t\treturn json.Marshal(struct {{ T string `json:\"{}\"`; {} }}{{ v.{}, v.{} }})", tag_json_name, variant.type_name, tag_field_name, variant.field_name)?;
                }
                writeln!(out, "\t}}")?;
                writeln!(out)?;
                writeln!(
                    out,
                    "\treturn nil, fmt.Errorf(\"bad {0} value: %s\", v.{0})",
                    tag_field_name
                )?;
                writeln!(out, "}}")?;

                writeln!(out)?;
                writeln!(out, "func (v *{}) UnmarshalJSON(b []byte) error {{", name)?;
                writeln!(
                    out,
                    "\tvar t struct {{ T string `json:\"{}\"` }}",
                    tag_json_name
                )?;
                writeln!(out, "\tif err := json.Unmarshal(b, &t); err != nil {{")?;
                writeln!(out, "\t\treturn err")?;
                writeln!(out, "\t}}")?;
                writeln!(out)?;
                writeln!(out, "\tvar err error")?;
                writeln!(out, "\tswitch t.T {{")?;
                for variant in &variants {
                    writeln!(out, "\tcase {:?}:", variant.tag_value)?;
                    writeln!(
                        out,
                        "\t\terr = json.Unmarshal(b, &v.{})",
                        variant.field_name
                    )?;
                }
                writeln!(out, "\tdefault:")?;
                writeln!(
                    out,
                    "\t\terr = fmt.Errorf(\"bad {} value: %s\", t.T)",
                    tag_field_name
                )?;
                writeln!(out, "\t}}")?;
                writeln!(out)?;
                writeln!(out, "\tif err != nil {{")?;
                writeln!(out, "\t\treturn err")?;
                writeln!(out, "\t}}")?;
                writeln!(out)?;
                writeln!(out, "\tv.{} = t.T", tag_field_name)?;
                writeln!(out, "\treturn nil")?;
                writeln!(out, "}}")?;

                None
            }

            target::Item::DiscriminatorVariant {
                metadata,
                name,
                fields,
                ..
            } => {
                if let Some(s) = metadata.get("goType").and_then(|v| v.as_str()) {
                    return Ok(Some(s.into()));
                }

                writeln!(out)?;
                write!(out, "{}", description(&metadata, 0))?;
                writeln!(out, "type {} struct {{", name)?;
                for (index, field) in fields.into_iter().enumerate() {
                    if index != 0 {
                        writeln!(out)?;
                    }

                    write!(out, "{}", description(&field.metadata, 1))?;

                    if field.optional {
                        writeln!(
                            out,
                            "\t{} {} `json:\"{},omitempty\"`",
                            field.name, field.type_, field.json_name
                        )?;
                    } else {
                        writeln!(
                            out,
                            "\t{} {} `json:\"{}\"`",
                            field.name, field.type_, field.json_name
                        )?;
                    }
                }
                writeln!(out, "}}")?;

                None
            }
        })
    }
}

#[derive(Default)]
pub struct FileState {
    imports: BTreeSet<String>,
}

fn description(metadata: &BTreeMap<String, Value>, indent: usize) -> String {
    doc(indent, jtd_codegen::target::metadata::description(metadata))
}

fn enum_variant_description(
    metadata: &BTreeMap<String, Value>,
    indent: usize,
    value: &str,
) -> String {
    doc(
        indent,
        jtd_codegen::target::metadata::enum_variant_description(metadata, value),
    )
}

fn doc(ident: usize, s: &str) -> String {
    let prefix = "\t".repeat(ident);
    jtd_codegen::target::fmt::comment_block("", &format!("{}// ", prefix), "", s)
}

#[cfg(test)]
mod tests {
    mod std_tests {
        jtd_codegen_test::std_test_cases!(&crate::Target::new("jtd_codegen_e2e".into()));
    }

    mod optional_std_tests {
        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into()),
            empty_and_nonascii_enum_values
        );
    }
}
