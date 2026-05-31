use std::fmt;

use super::ref_counted_set::RefCountedSet;
use super::size::{HyperlinkCountInt, OffsetInt, OffsetSlice};

pub(super) type Id = HyperlinkCountInt;
pub(super) type Set = RefCountedSet<PageEntry>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Hyperlink<'a> {
    pub(super) id: HyperlinkId<'a>,
    pub(super) uri: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HyperlinkId<'a> {
    Explicit(&'a [u8]),
    Implicit(OffsetInt),
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct PageEntry {
    id: PageEntryId,
    uri: OffsetSlice<u8>,
}

impl PageEntry {
    pub(super) const fn new(id: PageEntryId, uri: OffsetSlice<u8>) -> Self {
        Self { id, uri }
    }

    pub(super) const fn id(self) -> PageEntryId {
        self.id
    }

    pub(super) const fn uri(self) -> OffsetSlice<u8> {
        self.uri
    }
}

impl fmt::Debug for PageEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageEntry")
            .field("id", &self.id)
            .field("uri", &self.uri)
            .finish()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct PageEntryId {
    tag: PageEntryIdTag,
    _padding: [u8; 7],
    data: PageEntryIdData,
}

impl PageEntryId {
    pub(super) const fn implicit(value: OffsetInt) -> Self {
        Self {
            tag: PageEntryIdTag::Implicit,
            _padding: [0; 7],
            data: PageEntryIdData { implicit: value },
        }
    }

    pub(super) const fn explicit(value: OffsetSlice<u8>) -> Self {
        Self {
            tag: PageEntryIdTag::Explicit,
            _padding: [0; 7],
            data: PageEntryIdData { explicit: value },
        }
    }

    pub(super) const fn tag(self) -> PageEntryIdTag {
        self.tag
    }

    pub(super) fn implicit_value(self) -> OffsetInt {
        assert_eq!(self.tag, PageEntryIdTag::Implicit);
        unsafe {
            // Safety: tag checked above.
            self.data.implicit
        }
    }

    pub(super) fn explicit_value(self) -> OffsetSlice<u8> {
        assert_eq!(self.tag, PageEntryIdTag::Explicit);
        unsafe {
            // Safety: tag checked above.
            self.data.explicit
        }
    }
}

impl Default for PageEntryId {
    fn default() -> Self {
        Self::implicit(0)
    }
}

impl PartialEq for PageEntryId {
    fn eq(&self, other: &Self) -> bool {
        if self.tag != other.tag {
            return false;
        }

        match self.tag {
            PageEntryIdTag::Implicit => self.implicit_value() == other.implicit_value(),
            PageEntryIdTag::Explicit => self.explicit_value() == other.explicit_value(),
        }
    }
}

impl Eq for PageEntryId {}

impl fmt::Debug for PageEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tag {
            PageEntryIdTag::Implicit => f
                .debug_tuple("Implicit")
                .field(&self.implicit_value())
                .finish(),
            PageEntryIdTag::Explicit => f
                .debug_tuple("Explicit")
                .field(&self.explicit_value())
                .finish(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum PageEntryIdTag {
    #[default]
    Implicit = 0,
    Explicit = 1,
}

#[repr(C)]
#[derive(Clone, Copy)]
union PageEntryIdData {
    implicit: OffsetInt,
    explicit: OffsetSlice<u8>,
}
