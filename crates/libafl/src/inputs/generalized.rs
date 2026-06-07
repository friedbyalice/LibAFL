//! The `GeneralizedInput` is an input that ca be generalized to represent a rule, used by Grimoire

use alloc::{vec, vec::Vec};
use core::{
    borrow::Borrow,
    cell::UnsafeCell,
    hash::{Hash, Hasher},
};

use libafl_bolts::impl_serdeany;
use serde::{Deserialize, Serialize};

use crate::{
    Error, HasMetadata,
    corpus::Testcase,
    inputs::BytesInput,
    stages::mutational::{MutatedTransform, MutatedTransformPost},
    state::HasCorpus,
};

/// An item of the generalized input
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneralizedItem {
    /// Real bytes
    Bytes(Vec<u8>),
    /// An insertion point
    Gap,
}

/// Metadata regarding the generalised content of an input
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    any(not(feature = "serdeany_autoreg"), miri),
    expect(clippy::unsafe_derive_deserialize)
)] // for SerdeAny
pub struct GeneralizedInputMetadata {
    generalized: Vec<GeneralizedItem>,
    #[serde(skip)]
    bytes: UnsafeCell<BytesInput>,
}

impl Clone for GeneralizedInputMetadata {
    fn clone(&self) -> Self {
        Self {
            generalized: self.generalized.clone(),
            bytes: UnsafeCell::new(unsafe { (*self.bytes.get()).clone() }),
        }
    }
}

impl PartialEq for GeneralizedInputMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.generalized == other.generalized
    }
}

impl Eq for GeneralizedInputMetadata {}

impl Hash for GeneralizedInputMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.generalized.hash(state);
    }
}

impl Borrow<BytesInput> for GeneralizedInputMetadata {
    fn borrow(&self) -> &BytesInput {
        unsafe {
            *self.bytes.get() = BytesInput::from(compute_bytes(&self.generalized));
            &*self.bytes.get()
        }
    }
}

fn compute_bytes(generalized: &[GeneralizedItem]) -> Vec<u8> {
    generalized
        .iter()
        .filter_map(|item| match item {
            GeneralizedItem::Bytes(bytes) => Some(bytes),
            GeneralizedItem::Gap => None,
        })
        .flatten()
        .copied()
        .collect()
}

impl_serdeany!(GeneralizedInputMetadata);

impl GeneralizedInputMetadata {
    /// Fill the generalized vector from a slice of option (None -> Gap)
    #[must_use]
    pub fn generalized_from_options(v: &[Option<u8>]) -> Self {
        let mut generalized = vec![];
        let mut bytes = vec![];
        if v.first() != Some(&None) {
            generalized.push(GeneralizedItem::Gap);
        }
        for e in v {
            match e {
                None => {
                    if !bytes.is_empty() {
                        generalized.push(GeneralizedItem::Bytes(bytes.clone()));
                        bytes.clear();
                    }
                    generalized.push(GeneralizedItem::Gap);
                }
                Some(b) => {
                    bytes.push(*b);
                }
            }
        }
        if !bytes.is_empty() {
            generalized.push(GeneralizedItem::Bytes(bytes));
        }
        if generalized.last() != Some(&GeneralizedItem::Gap) {
            generalized.push(GeneralizedItem::Gap);
        }
        let bytes_bytes = compute_bytes(&generalized);
        Self {
            generalized,
            bytes: UnsafeCell::new(BytesInput::from(bytes_bytes)),
        }
    }

    /// Get the size of the generalized
    #[must_use]
    pub fn generalized_len(&self) -> usize {
        let mut size = 0;
        for item in &self.generalized {
            match item {
                GeneralizedItem::Bytes(b) => size += b.len(),
                GeneralizedItem::Gap => size += 1,
            }
        }
        size
    }

    /// Convert generalized to bytes
    #[must_use]
    pub fn generalized_to_bytes(&self) -> Vec<u8> {
        compute_bytes(&self.generalized)
    }

    /// Get the generalized input
    #[must_use]
    pub fn generalized(&self) -> &[GeneralizedItem] {
        &self.generalized
    }

    /// Get the generalized input (mutable)
    pub fn generalized_mut(&mut self) -> &mut Vec<GeneralizedItem> {
        &mut self.generalized
    }
}

impl<S> MutatedTransform<BytesInput, S> for GeneralizedInputMetadata
where
    S: HasCorpus<BytesInput>,
{
    type Post = Self;

    fn try_transform_from(base: &mut Testcase<BytesInput>, state: &S) -> Result<Self, Error> {
        let input = base.load_input(state.corpus())?.clone();
        let meta = base
            .metadata_map()
            .get::<GeneralizedInputMetadata>()
            .ok_or_else(|| {
                Error::key_not_found(format!(
                    "Couldn't find the GeneralizedInputMetadata for corpus entry {base:?}",
                ))
            })
            .cloned()?;
        let result = meta;
        unsafe {
            *result.bytes.get() = input;
        }
        Ok(result)
    }

    fn try_transform_into(self, _state: &S) -> Result<(BytesInput, Self::Post), Error> {
        Ok((BytesInput::from(compute_bytes(&self.generalized)), self))
    }
}

impl<S> MutatedTransformPost<S> for GeneralizedInputMetadata {}
