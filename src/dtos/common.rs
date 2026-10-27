use loco_rs::model::query::PageResponse;
use ts_rs::TS;

/// A page of results: the items, plus the framework's pagination metadata
/// flattened alongside them.
///
/// The field names are exactly `PagerMeta`'s (`page`, `page_size`,
/// `total_pages`, `total_items`), so an app has one pagination vocabulary
/// whether a handler answers with this typed envelope or with the framework's
/// own `Pager`. Build it with [`Page::from_query`] rather than by hand — that
/// is what keeps the two in step.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct Page<T: TS> {
    pub items: Vec<T>,
    #[ts(type = "number")]
    pub page: u64,
    #[ts(type = "number")]
    pub page_size: u64,
    #[ts(type = "number")]
    pub total_pages: u64,
    #[ts(type = "number")]
    pub total_items: u64,
}

impl<T: TS> Page<T> {
    /// Build a page from a paginated query (`query::paginate`,
    /// `query::fetch_page`), mapping each model onto its DTO.
    pub fn from_query<M>(res: PageResponse<M>) -> Self
    where
        T: From<M>,
    {
        Self {
            items: res.page.into_iter().map(T::from).collect(),
            page: res.meta.page,
            page_size: res.meta.page_size,
            total_pages: res.meta.total_pages,
            total_items: res.meta.total_items,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[ts(type = "unknown")]
    pub details: Option<serde_json::Value>,
}

/// One field of a partial update: left out (`Keep`), `null` (`Clear`) or a
/// value (`Set`). Plain `Option<T>` cannot tell the first two apart.
///
/// Mark the field `#[serde(default, skip_serializing_if = "Patch::is_keep")]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Patch<T> {
    #[default]
    Keep,
    Clear,
    Set(T),
}

impl<T> Patch<T> {
    #[must_use]
    pub const fn is_keep(&self) -> bool {
        matches!(self, Self::Keep)
    }

    /// `None` to keep, `Some(None)` to clear, `Some(Some(v))` to set.
    pub fn into_option(self) -> Option<Option<T>> {
        match self {
            Self::Keep => None,
            Self::Clear => Some(None),
            Self::Set(v) => Some(Some(v)),
        }
    }
}

/// A CLI flag: given means set, absent means keep.
impl<T> From<Option<T>> for Patch<T> {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Keep, Self::Set)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Patch<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <Option<T> as serde::Deserialize>::deserialize(deserializer)
            .map(|v| v.map_or(Self::Clear, Self::Set))
    }
}

impl<T: serde::Serialize> serde::Serialize for Patch<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Set(v) => v.serialize(serializer),
            Self::Keep | Self::Clear => serializer.serialize_none(),
        }
    }
}
