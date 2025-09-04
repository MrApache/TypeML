pub struct Spanned;

pub enum CstNode {
    Directive(Directive),
    Enum(Enum),
}

pub struct Directive {
    hash: Spanned,
    keyword: Spanned,
    content: Spanned,
}

pub struct Enum {
    keyword: Spanned,
    identifier: Spanned,
    lcb: Spanned,
    variants: Vec<EnumVariant>,
    rcb: Spanned,
}

pub struct EnumVariant {
    identifier: Spanned,
    properties: Option<EnumPipeInstruction>,
    comma: Option<Spanned>,
}

pub struct EnumPipeInstruction {
    pipe: Spanned,
    value: Option<Spanned>,
}

pub struct TypeDef {
    keyword: Spanned,
    identifier: Spanned,
    kind: TypeDefKind,
}

pub struct EmptyTypeDef {
    semicolon: Spanned,
}

pub struct EmptyWithBindTypeDef {
    operator: Spanned,
    identifier: Spanned,
    semicolon: Spanned,
}

pub struct TypeDefWithBody {
    lcb: Spanned,
    fields: Vec<Field>,
    rcb: Spanned,
}

pub struct FullTypeDef {
    operator: Spanned,
    identifier: Spanned,
    lcb: Spanned,
    fields: Vec<Field>,
    rcb: Spanned,
}

pub struct Field {
    identifier: Spanned,
    colon: Spanned,
    ty: Spanned,
}

pub enum TypeDefKind {
    Empty(EmptyTypeDef),
    EmptyWithBind(EmptyWithBindTypeDef),
    WithBody(TypeDefWithBody),
    Full(FullTypeDef),
}
