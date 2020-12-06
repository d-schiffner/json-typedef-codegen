use crate::{Inflector, Result};
use std::io::Write;

pub trait Target {
    type FileState: Default;
    type ExprMeta: ExprMeta;

    fn file_partitioning(&self) -> FilePartitioning;
    fn enum_strategy(&self) -> EnumStrategy;

    fn name_type(&self, name_parts: &[String]) -> String;
    fn name_field(&self, name_parts: &[String]) -> String;
    fn name_enum_variant(&self, name_parts: &[String]) -> String;

    fn boolean(&self, state: &mut Self::FileState) -> Expr<Self::ExprMeta>;
    fn string(&self, state: &mut Self::FileState) -> Expr<Self::ExprMeta>;

    fn nullable_of(
        &self,
        state: &mut Self::FileState,
        expr: Expr<Self::ExprMeta>,
    ) -> Expr<Self::ExprMeta>;

    fn elements_of(
        &self,
        state: &mut Self::FileState,
        expr: Expr<Self::ExprMeta>,
    ) -> Expr<Self::ExprMeta>;

    fn write_preamble<'a>(&self, state: &mut Self::FileState, out: &mut dyn Write) -> Result<()>;

    fn write_alias<'a>(
        &self,
        state: &mut Self::FileState,
        out: &mut dyn Write,
        alias: Alias<Self::ExprMeta>,
    ) -> Result<Expr<Self::ExprMeta>>;

    fn write_enum(
        &self,
        state: &mut Self::FileState,
        out: &mut dyn Write,
        enum_: Enum,
    ) -> Result<Expr<Self::ExprMeta>>;

    fn write_enum_variant(
        &self,
        state: &mut Self::FileState,
        out: &mut dyn Write,
        enum_variant: EnumVariant,
    ) -> Result<Expr<Self::ExprMeta>>;

    fn write_struct<'a>(
        &self,
        state: &mut Self::FileState,
        out: &mut dyn Write,
        struct_: Struct<Self::ExprMeta>,
    ) -> Result<Expr<Self::ExprMeta>>;
    // fn write_discriminator_variant<'a>(
    //     &self,
    //     state: &mut Self::FileState,
    //     out: &mut dyn Write,
    //     discriminator_variant: DiscriminatorVariant<'a, Self::Expr>,
    // ) -> Result<()>;
    // fn write_discriminator<'a>(
    //     &self,
    //     state: &mut Self::FileState,
    //     out: &mut dyn Write,
    //     discriminator: Discriminator<'a, Self::Expr>,
    // ) -> Result<()>;
}

#[derive(Clone)]
pub struct Expr<T> {
    pub expr: String,
    pub meta: T,
}

pub trait ExprMeta: PartialEq + Clone {
    fn universally_usable() -> Self;
}

pub enum FilePartitioning {
    SingleFile(String),
    FilePerType(String),
}

pub enum EnumStrategy {
    Modularized,
    Unmodularized,
}

pub struct Alias<T> {
    pub name: String,
    pub description: String,
    pub type_: Expr<T>,
}

pub struct Enum {
    pub name: String,
    pub description: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone)]
pub struct EnumVariant {
    pub name: String,
    pub description: String,
    pub json_value: String,
}

pub struct Struct<T> {
    pub name: String,
    pub description: String,
    pub has_additional: bool,
    pub fields: Vec<StructField<T>>,
}

pub struct StructField<T> {
    pub name: String,
    pub json_name: String,
    pub description: String,
    pub optional: bool,
    pub type_: Expr<T>,
}

// pub struct DiscriminatorVariant<'a, T> {
//     pub name: String,
//     pub description: &'a str,
//     pub parent_name: &'a str,
//     pub discriminator_value: &'a str,
//     pub struct_: Struct<'a, T>,
// }

// pub struct Discriminator<'a, T> {
//     pub name: String,
//     pub description: &'a str,
//     pub discriminator_field_name: &'a str,
//     pub discriminator_json_name: &'a str,
//     pub struct_: Struct<'a, T>,
// }
