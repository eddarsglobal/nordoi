use super::ast::{AstElement, AstFile};
use super::lexer::{Token, TokenKind};
use super::module_error::{ModuleError, ModuleResult};
use super::name::Name;
use super::parser::parse;
use super::source::{SourceSpan, SourceText};

pub const MAX_MODULE_SEGMENTS: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    span: SourceSpan,
    segments: Vec<Name>,
}

impl ModulePath {
    pub(crate) fn new(span: SourceSpan, segments: Vec<Name>) -> Self {
        Self { span, segments }
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn segments(&self) -> &[Name] {
        &self.segments
    }

    pub fn canonical_text(&self, source: &SourceText) -> ModuleResult<String> {
        let mut output = String::new();
        for (index, segment) in self.segments.iter().enumerate() {
            if index != 0 {
                output.push('.');
            }
            output.push_str(segment.text(source)?);
        }
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDecl {
    span: SourceSpan,
    keyword: Token,
    path: ModulePath,
    terminator: Token,
}

impl ModuleDecl {
    pub(crate) fn new(
        span: SourceSpan,
        keyword: Token,
        path: ModulePath,
        terminator: Token,
    ) -> Self {
        Self {
            span,
            keyword,
            path,
            terminator,
        }
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    pub const fn keyword(&self) -> &Token {
        &self.keyword
    }

    pub const fn path(&self) -> &ModulePath {
        &self.path
    }

    pub const fn terminator(&self) -> &Token {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleUnit {
    file: AstFile,
    module: Option<ModuleDecl>,
}

impl ModuleUnit {
    pub(crate) fn new(file: AstFile, module: Option<ModuleDecl>) -> Self {
        Self { file, module }
    }

    pub const fn file(&self) -> &AstFile {
        &self.file
    }

    pub fn module(&self) -> Option<&ModuleDecl> {
        self.module.as_ref()
    }

    pub const fn is_anonymous(&self) -> bool {
        self.module.is_none()
    }
}

pub fn analyze_module_unit(source: &SourceText) -> ModuleResult<ModuleUnit> {
    ModuleAnalyzer::new(source)?.analyze()
}

#[derive(Debug)]
pub struct ModuleAnalyzer<'a> {
    source: &'a SourceText,
    file: AstFile,
}

impl<'a> ModuleAnalyzer<'a> {
    pub fn new(source: &'a SourceText) -> ModuleResult<Self> {
        Ok(Self {
            source,
            file: parse(source)?,
        })
    }

    pub fn analyze(self) -> ModuleResult<ModuleUnit> {
        let declaration = self.parse_leading_module_decl()?;
        Ok(ModuleUnit::new(self.file, declaration))
    }

    fn parse_leading_module_decl(&self) -> ModuleResult<Option<ModuleDecl>> {
        let mut cursor = ElementCursor::new(self.file.elements());
        let Some(first) = cursor.next_significant() else {
            return Ok(None);
        };

        let AstElement::Token(keyword) = first else {
            return Ok(None);
        };

        if keyword.kind() != &TokenKind::Identifier
            || self.source.slice(keyword.span())? != "module"
        {
            return Ok(None);
        }

        let name_element = cursor
            .next_significant()
            .ok_or(ModuleError::ExpectedModuleName {
                span: self.file.eof().span(),
            })?;
        let first_name = self.name_from_element(name_element, None)?;

        let mut segments = vec![first_name];
        let mut last_name_span = first_name.span();

        loop {
            let Some(element) = cursor.next_significant() else {
                return Err(ModuleError::MissingModuleTerminator {
                    span: self.file.eof().span(),
                });
            };

            let AstElement::Token(token) = element else {
                return Err(ModuleError::ExpectedModulePathSeparatorOrTerminator {
                    span: element.span(),
                });
            };

            match token.kind() {
                TokenKind::Punctuation(';') => {
                    let path_span = SourceSpan::new_unchecked(
                        self.source.id(),
                        first_name.span().start().get(),
                        last_name_span.end().get(),
                    );
                    let declaration_span = SourceSpan::new_unchecked(
                        self.source.id(),
                        keyword.span().start().get(),
                        token.span().end().get(),
                    );
                    return Ok(Some(ModuleDecl::new(
                        declaration_span,
                        keyword.clone(),
                        ModulePath::new(path_span, segments),
                        token.clone(),
                    )));
                }
                TokenKind::Punctuation('.') => {
                    let separator_span = token.span();
                    let next = cursor.next_significant().ok_or(
                        ModuleError::ExpectedModuleNameAfterSeparator {
                            separator_span,
                            span: self.file.eof().span(),
                        },
                    )?;
                    let name = self.name_from_element(next, Some(separator_span))?;
                    if segments.len() as u32 >= MAX_MODULE_SEGMENTS {
                        return Err(ModuleError::TooManyModuleSegments {
                            maximum: MAX_MODULE_SEGMENTS,
                            span: name.span(),
                        });
                    }
                    last_name_span = name.span();
                    segments.push(name);
                }
                _ => {
                    return Err(ModuleError::ExpectedModulePathSeparatorOrTerminator {
                        span: token.span(),
                    });
                }
            }
        }
    }

    fn name_from_element(
        &self,
        element: &AstElement,
        separator_span: Option<SourceSpan>,
    ) -> ModuleResult<Name> {
        let AstElement::Token(token) = element else {
            return Err(match separator_span {
                Some(separator_span) => ModuleError::ExpectedModuleNameAfterSeparator {
                    separator_span,
                    span: element.span(),
                },
                None => ModuleError::ExpectedModuleName {
                    span: element.span(),
                },
            });
        };

        if token.kind() != &TokenKind::Identifier {
            return Err(match separator_span {
                Some(separator_span) => ModuleError::ExpectedModuleNameAfterSeparator {
                    separator_span,
                    span: token.span(),
                },
                None => ModuleError::ExpectedModuleName { span: token.span() },
            });
        }

        Ok(Name::from_token(self.source, token)?)
    }
}

#[derive(Debug)]
struct ElementCursor<'a> {
    elements: &'a [AstElement],
    index: usize,
}

impl<'a> ElementCursor<'a> {
    fn new(elements: &'a [AstElement]) -> Self {
        Self { elements, index: 0 }
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
