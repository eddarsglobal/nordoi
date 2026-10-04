use super::error::{SourceError, SourceResult};

pub const MAX_SOURCE_BYTES: u64 = (u32::MAX - 1) as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(u32);

impl SourceId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(u32);

impl ByteOffset {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    source: SourceId,
    start: ByteOffset,
    end: ByteOffset,
}

impl SourceSpan {
    pub const fn source(self) -> SourceId {
        self.source
    }

    pub const fn start(self) -> ByteOffset {
        self.start
    }

    pub const fn end(self) -> ByteOffset {
        self.end
    }

    pub const fn len(self) -> u32 {
        self.end.0 - self.start.0
    }

    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }

    pub(crate) const fn new_unchecked(source: SourceId, start: u32, end: u32) -> Self {
        Self {
            source,
            start: ByteOffset(start),
            end: ByteOffset(end),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    source: SourceId,
    offset: ByteOffset,
    line: u32,
    column: u32,
}

impl SourcePosition {
    pub const fn source(self) -> SourceId {
        self.source
    }

    pub const fn offset(self) -> ByteOffset {
        self.offset
    }

    /// One-based physical line number.
    pub const fn line(self) -> u32 {
        self.line
    }

    /// One-based Unicode-scalar column number within the physical line.
    pub const fn column(self) -> u32 {
        self.column
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceText {
    id: SourceId,
    name: String,
    text: String,
    line_starts: Vec<u32>,
}

impl SourceText {
    /// Creates one source unit. `SourceId` is compiler/caller-issued and must be unique
    /// among source units whose spans may be mixed in the same frontend session.
    pub fn new(
        id: SourceId,
        name: impl Into<String>,
        text: impl Into<String>,
    ) -> SourceResult<Self> {
        let text = text.into();
        let bytes = text.len() as u64;
        if bytes > MAX_SOURCE_BYTES {
            return Err(SourceError::SourceTooLarge {
                bytes,
                maximum: MAX_SOURCE_BYTES,
            });
        }

        let mut line_starts = vec![0];
        let bytes = text.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            match bytes[index] {
                b'\r' => {
                    index += 1;
                    if index < bytes.len() && bytes[index] == b'\n' {
                        index += 1;
                    }
                    line_starts.push(index as u32);
                }
                b'\n' => {
                    index += 1;
                    line_starts.push(index as u32);
                }
                _ => index += 1,
            }
        }

        Ok(Self {
            id,
            name: name.into(),
            text,
            line_starts,
        })
    }

    pub const fn id(&self) -> SourceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> ByteOffset {
        ByteOffset(self.text.len() as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    pub fn full_span(&self) -> SourceSpan {
        SourceSpan::new_unchecked(self.id, 0, self.text.len() as u32)
    }

    pub fn span(&self, start: ByteOffset, end: ByteOffset) -> SourceResult<SourceSpan> {
        self.validate_offset(start)?;
        self.validate_offset(end)?;
        if start > end {
            return Err(SourceError::InvalidSpanOrder { start, end });
        }
        Ok(SourceSpan {
            source: self.id,
            start,
            end,
        })
    }

    pub fn slice(&self, span: SourceSpan) -> SourceResult<&str> {
        if span.source != self.id {
            return Err(SourceError::SourceMismatch {
                expected: self.id,
                actual: span.source,
            });
        }
        self.validate_offset(span.start)?;
        self.validate_offset(span.end)?;
        if span.start > span.end {
            return Err(SourceError::InvalidSpanOrder {
                start: span.start,
                end: span.end,
            });
        }
        Ok(&self.text[span.start.0 as usize..span.end.0 as usize])
    }

    pub fn position(&self, offset: ByteOffset) -> SourceResult<SourcePosition> {
        self.validate_offset(offset)?;
        let raw = offset.0;
        let line_index = self
            .line_starts
            .partition_point(|line_start| *line_start <= raw)
            .saturating_sub(1);
        let line_start = self.line_starts[line_index] as usize;
        let offset_usize = raw as usize;
        let column = self.text[line_start..offset_usize].chars().count() as u32 + 1;

        Ok(SourcePosition {
            source: self.id,
            offset,
            line: line_index as u32 + 1,
            column,
        })
    }

    fn validate_offset(&self, offset: ByteOffset) -> SourceResult<()> {
        let length = self.len();
        if offset > length {
            return Err(SourceError::OffsetOutOfBounds {
                offset,
                source_len: length,
            });
        }
        if !self.text.is_char_boundary(offset.0 as usize) {
            return Err(SourceError::OffsetNotCharBoundary { offset });
        }
        Ok(())
    }
}
