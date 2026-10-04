use super::ast::AstElement;
use super::lexer::{Token, TokenKind};
use super::module::{analyze_module_unit, ModuleUnit};
use super::name::Name;
use super::source::{SourceSpan, SourceText};
use super::type_effect_error::{TypeEffectError, TypeEffectResult};

pub const MAX_TYPE_EFFECT_DECLARATIONS: u32 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceDeclarationKind {
    Type,
    Effect,
}

impl SurfaceDeclarationKind {
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Effect => "effect",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceDeclaration {
    kind: SurfaceDeclarationKind,
    span: SourceSpan,
    keyword: Token,
    name: Name,
    terminator: Token,
}

impl SurfaceDeclaration {
    pub(crate) fn new(
        kind: SurfaceDeclarationKind,
        span: SourceSpan,
        keyword: Token,
        name: Name,
        terminator: Token,
    ) -> Self {
        Self {
            kind,
            span,
            keyword,
            name,
            terminator,
        }
    }

    pub fn kind(&self) -> SurfaceDeclarationKind {
        self.kind
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn keyword(&self) -> &Token {
        &self.keyword
    }

    pub fn name(&self) -> Name {
        self.name
    }

    pub fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEffectUnit {
    module_unit: ModuleUnit,
    declarations: Vec<SurfaceDeclaration>,
    body_span: SourceSpan,
}

impl TypeEffectUnit {
    pub(crate) fn new(
        module_unit: ModuleUnit,
        declarations: Vec<SurfaceDeclaration>,
        body_span: SourceSpan,
    ) -> Self {
        Self {
            module_unit,
            declarations,
            body_span,
        }
    }

    pub fn module_unit(&self) -> &ModuleUnit {
        &self.module_unit
    }

    pub fn declarations(&self) -> &[SurfaceDeclaration] {
        &self.declarations
    }

    pub fn body_span(&self) -> SourceSpan {
        self.body_span
    }
}

pub fn analyze_type_effect_unit(source: &SourceText) -> TypeEffectResult<TypeEffectUnit> {
    let module_unit = analyze_module_unit(source)?;
    TypeEffectAnalyzer::new(source, module_unit).analyze()
}

#[derive(Debug)]
pub struct TypeEffectAnalyzer<'a> {
    source: &'a SourceText,
    module_unit: ModuleUnit,
}

impl<'a> TypeEffectAnalyzer<'a> {
    pub fn new(source: &'a SourceText, module_unit: ModuleUnit) -> Self {
        Self {
            source,
            module_unit,
        }
    }

    pub fn analyze(self) -> TypeEffectResult<TypeEffectUnit> {
        let file_span = self.module_unit.file().span();
        let mut body_start = self
            .module_unit
            .module()
            .map(|declaration| declaration.span().end())
            .unwrap_or(file_span.start());
        let mut cursor =
            ElementCursor::after_offset(self.module_unit.file().elements(), body_start.get());
        let mut declarations = Vec::new();

        while let Some(element) = cursor.next_significant() {
            let Some(kind) = self.declaration_kind(element)? else {
                break;
            };

            if declarations.len() as u32 >= MAX_TYPE_EFFECT_DECLARATIONS {
                return Err(TypeEffectError::TooManyDeclarations {
                    maximum: MAX_TYPE_EFFECT_DECLARATIONS,
                    span: element.span(),
                });
            }

            let keyword = element
                .as_token()
                .expect("declaration kind is recognized only from a token")
                .clone();
            let name_element = match cursor.next_significant() {
                Some(element) => element,
                None => {
                    return Err(
                        self.expected_name_error(kind, self.module_unit.file().eof().span())
                    );
                }
            };
            let name = self.name_from_element(kind, name_element)?;
            let terminator_element = match cursor.next_significant() {
                Some(element) => element,
                None => {
                    return Err(
                        self.expected_terminator_error(kind, self.module_unit.file().eof().span())
                    );
                }
            };
            let terminator = self.terminator_from_element(kind, terminator_element)?;
            let declaration_span = self
                .source
                .span(keyword.span().start(), terminator.span().end())?;
            body_start = terminator.span().end();
            declarations.push(SurfaceDeclaration::new(
                kind,
                declaration_span,
                keyword,
                name,
                terminator,
            ));
        }

        let body_span = self.source.span(body_start, file_span.end())?;
        Ok(TypeEffectUnit::new(
            self.module_unit,
            declarations,
            body_span,
        ))
    }

    fn declaration_kind(
        &self,
        element: &AstElement,
    ) -> TypeEffectResult<Option<SurfaceDeclarationKind>> {
        let Some(token) = element.as_token() else {
            return Ok(None);
        };
        if token.kind() != &TokenKind::Identifier {
            return Ok(None);
        }
        let text = self.source.slice(token.span())?;
        Ok(match text {
            "type" => Some(SurfaceDeclarationKind::Type),
            "effect" => Some(SurfaceDeclarationKind::Effect),
            _ => None,
        })
    }

    fn name_from_element(
        &self,
        kind: SurfaceDeclarationKind,
        element: &AstElement,
    ) -> TypeEffectResult<Name> {
        let Some(token) = element.as_token() else {
            return Err(self.expected_name_error(kind, element.span()));
        };
        if token.kind() != &TokenKind::Identifier {
            return Err(self.expected_name_error(kind, token.span()));
        }
        Ok(Name::from_token(self.source, token)?)
    }

    fn terminator_from_element(
        &self,
        kind: SurfaceDeclarationKind,
        element: &AstElement,
    ) -> TypeEffectResult<Token> {
        let Some(token) = element.as_token() else {
            return Err(self.expected_terminator_error(kind, element.span()));
        };
        if token.kind() != &TokenKind::Punctuation(';') {
            return Err(self.expected_terminator_error(kind, token.span()));
        }
        Ok(token.clone())
    }

    fn expected_name_error(
        &self,
        kind: SurfaceDeclarationKind,
        span: SourceSpan,
    ) -> TypeEffectError {
        match kind {
            SurfaceDeclarationKind::Type => TypeEffectError::ExpectedTypeName { span },
            SurfaceDeclarationKind::Effect => TypeEffectError::ExpectedEffectName { span },
        }
    }

    fn expected_terminator_error(
        &self,
        kind: SurfaceDeclarationKind,
        span: SourceSpan,
    ) -> TypeEffectError {
        match kind {
            SurfaceDeclarationKind::Type => TypeEffectError::ExpectedTypeTerminator { span },
            SurfaceDeclarationKind::Effect => TypeEffectError::ExpectedEffectTerminator { span },
        }
    }
}

#[derive(Debug)]
struct ElementCursor<'a> {
    elements: &'a [AstElement],
    index: usize,
}

impl<'a> ElementCursor<'a> {
    fn after_offset(elements: &'a [AstElement], offset: u32) -> Self {
        let mut index = 0usize;
        while index < elements.len() && elements[index].span().end().get() <= offset {
            index += 1;
        }
        Self { elements, index }
    }

    fn next_significant(&mut self) -> Option<&'a AstElement> {
        while self.index < self.elements.len() {
            let element = &self.elements[self.index];
            self.index += 1;
            match element {
                AstElement::Token(token) if token.is_trivia() => continue,
                _ => return Some(element),
            }
        }
        None
    }
}
