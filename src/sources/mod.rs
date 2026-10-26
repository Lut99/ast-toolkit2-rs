//  SOURCES.rs
//    by Lut99
//
//  Description:
//!   Provides an abstract [`Sources`] collection that expertises in managing
//!   source texts as an AST is parsed and later errors are handled.
//

use std::borrow::Cow;
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs::File;
use std::hash::{BuildHasher, RandomState};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use hashbrown::HashTable;
use polonius_the_crab::{polonius, polonius_return};

use crate::loc::Range;


/***** ERRORS *****/
/// Defines errors for the [`TextRenderer`].
#[derive(Debug)]
pub enum Error {
    /// Failed to open the given file.
    FileOpen { path: PathBuf, err: std::io::Error },
    /// Failed to read from a given reader.
    ReaderRead { name: String, err: std::io::Error },
    /// Failed to seek in a given reader.
    ReaderSeek { name: String, err: std::io::Error },
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::FileOpen { path, .. } => write!(f, "Failed to open file {path:?}"),
            Self::ReaderRead { name, .. } => write!(f, "Failed to read from reader {name:?}"),
            Self::ReaderSeek { name, .. } => write!(f, "Failed to seek in reader {name:?}"),
        }
    }
}
impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::FileOpen { err, .. } => Some(err),
            Self::ReaderRead { err, .. } => Some(err),
            Self::ReaderSeek { err, .. } => Some(err),
        }
    }
}





/***** HELPER TRAITS *****/
/// Something [`Seek`]- ánd [`Read`]able.
pub trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}





/***** HELPERS *****/
/// Organizes a source entry.
struct Source<'r> {
    /// The ID of this source.
    id:     u64,
    /// The name of this source.
    name:   String,
    /// The length of this source.
    len:    u64,
    /// The reader of this source.
    reader: Box<dyn 'r + SeekRead>,
    /// If we've cached a part of it (and which part).
    cache:  Option<(Range, Cow<'r, [u8]>)>,
}

impl<'r> Source<'r> {
    /// Compares this source's ID with the given hash, but delayed.
    ///
    /// # Arguments
    /// - `hash`: Some hash to compare with this ID.
    ///
    /// # Returns
    /// A [`FnOnce`] that does the comparison.
    #[inline]
    const fn is(hash: u64) -> impl for<'s> FnMut(&'s Self) -> bool { move |s: &Self| -> bool { s.id == hash } }

    /// Returns the "hash" (=ID) of this Source.
    ///
    /// # Returns
    /// A number encoding the hash, which happens to be the ID.
    #[inline]
    const fn hash(&self) -> u64 { self.id }
}





/***** LIBRARY *****/
/// Collection of (byte) source texts that can be collected during [parsing](super::nibble) or
/// used to [show errors](super::diag).
///
/// Usually, you want to use this struct in combination with the `nibble`- and `diag`-features.
pub struct Sources<'r, H = RandomState> {
    /// A state to hash IDs with.
    state:   H,
    /// A map of IDs to their respective names and readers.
    ///
    /// Note that, for membership, this map is leading!
    ///
    /// The hash is cached to avoid having to recompute it the whole time.
    sources: HashTable<Source<'r>>,
}

// Constructors
impl<'r> Default for Sources<'r, RandomState> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'r> Sources<'r, RandomState> {
    /// Constructor for the Sources that initializes it as empty.
    ///
    /// # Returns
    /// A new Sources that has little memory allocated but is ready to accept the first.
    #[inline]
    pub fn new() -> Self { Self { state: RandomState::new(), sources: HashTable::new() } }

    /// Constructor for the Sources that initializes it as empty but with already some memory allocated.
    ///
    /// # Arguments
    /// - `capacity`: An initial number of sources for which this Sources should reserve memory.
    ///   Note that it may reserve more if the backing allocator wants to.
    ///
    /// # Returns
    /// A new Sources that has memory allocated for at least `capacity` sources, but is otherwise empty.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { state: RandomState::new(), sources: HashTable::with_capacity(capacity) } }
}
impl<'r, H> Sources<'r, H> {
    /// Constructor for the Sources that initializes it as empty but with the given `H`asher.
    ///
    /// # Arguments
    /// - `state`: Some [`BuildHasher`] to use to compute hashes.
    ///
    /// # Returns
    /// A new Sources that has little memory allocated but is ready to accept the first.
    #[inline]
    pub const fn with_hasher(state: H) -> Self { Self { state, sources: HashTable::new() } }

    /// Constructor for the Sources that initializes it as empty but with already some memory
    /// allocated. And the given hasher.
    ///
    /// # Arguments
    /// - `capacity`: An initial number of sources for which this Sources should reserve memory.
    ///   Note that it may reserve more if the backing allocator wants to.
    /// - `state`: Some [`BuildHasher`] to use to compute hashes.
    ///
    /// # Returns
    /// A new Sources that has memory allocated for at least `capacity` sources, but is otherwise empty.
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, state: H) -> Self { Self { state, sources: HashTable::with_capacity(capacity) } }
}

// Collection - mutation
impl<'r, H: BuildHasher> Sources<'r, H> {
    /// Adds a new source.
    ///
    /// The source must be something that is both [`Read`]able as [`Seek`]able. For example, an
    /// [in-memory string](Sources::add_str()) or [a file](Sources::add_file()) are often-used
    /// examples of this.
    ///
    /// # Arguments
    /// - `name`: Some kind of name for this source. Note that its hash will be used as internal
    ///   ID!
    /// - `source`: The actual source to add.
    /// - `len`: The total length of the source, in bytes.
    ///
    /// # Returns
    /// The ID for this source.
    ///
    /// # Panics
    /// This function panics if something under that `name` was already added.
    #[track_caller]
    pub fn add(&mut self, name: impl Into<String>, source: impl 'r + Seek + Read, len: u64) -> u64 {
        // First, hash the name
        let name: String = name.into();
        let id: u64 = self.state.hash_one(&name);

        // Check if we already know it
        if self.sources.find(id, Source::is(id)).is_some() {
            panic!("Cannot add sources with the same name {name:?}");
        }

        // Compute the length of the reader
        let reader: Box<dyn SeekRead> = Box::new(source);
        self.sources.insert_unique(id, Source { id, name, len, reader, cache: None }, Source::hash);
        id
    }

    /// Adds a byte array (reference) as source to the TextRenderer.
    ///
    /// If you want to add something more esotheric, consider calling [`TextRenderer::add()`].
    ///
    /// # Arguments
    /// - `name`: Some kind of name for this source. Note that its hash will be used as internal
    ///   ID!
    /// - `source`: The byte array to add.
    ///
    /// # Returns
    /// The ID for this source.
    ///
    /// # Panics
    /// This function panics if something under that `name` was already added.
    #[inline]
    #[track_caller]
    pub fn add_bytes(&mut self, name: impl Into<String>, source: &'r [u8]) -> u64 {
        let id: u64 = self.add(name, Cursor::new(source), source.len() as u64);
        // Also set a cache
        if let Some(s) = self.sources.find_mut(id, Source::is(id)) {
            s.cache = Some((Range::full().contextualize(source.len() as u64), Cow::Borrowed(source)));
        }
        id
    }

    /// Adds a string (reference) as source to the TextRenderer.
    ///
    /// If you want to add something more esotheric, consider calling [`TextRenderer::add()`].
    ///
    /// # Arguments
    /// - `name`: Some kind of name for this source. Note that its hash will be used as internal
    ///   ID!
    /// - `source`: The string to add.
    ///
    /// # Returns
    /// The ID for this source.
    ///
    /// # Panics
    /// This function panics if something under that `name` was already added.
    #[inline]
    #[track_caller]
    pub fn add_str(&mut self, name: impl Into<String>, source: &'r str) -> u64 { self.add_bytes(name, source.as_bytes()) }

    /// Adds a file as source to the TextRenderer.
    ///
    /// If you want to add something more esotheric, consider calling [`TextRenderer::add()`].
    ///
    /// # Arguments
    /// - `name`: Some kind of name for this source. Note that its hash will be used as internal
    ///   ID!
    /// - `path`: The path to the file to add.
    ///
    /// # Returns
    /// The ID for this source.
    ///
    /// # Errors
    /// This function fails if it fails to open the given `path` as a file.
    ///
    /// # Panics
    /// This function panics if something under that `name` was already added.
    #[inline]
    #[track_caller]
    pub fn add_file(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> Result<u64, Error> {
        // Open a file handle first
        let path: &Path = path.as_ref();
        let mut handle: File = File::open(path).map_err(|err| Error::FileOpen { path: path.into(), err })?;

        // Find its length
        let len: u64 = handle.seek(SeekFrom::End(0)).map_err(|err| Error::ReaderSeek { name: path.to_string_lossy().into(), err })?;

        // Then call the addition function
        Ok(self.add(name, handle, len))
    }



    /// Returns a reference to (a slice of a) source text in the Sources.
    ///
    /// If a source is given as a [byte array](Sources::add_bytes()) or
    /// [string](Sources::add_str()), then this function simply returns pointers to those buffers.
    /// Else, the associated is called and the source text is loaded in an internal cache before
    /// returning pointers.
    ///
    /// # Arguments
    /// - `id`: The ID of the source text to load.
    /// - `range`: Some [`Range`] to load from a source text. Use [`Range::full()`] if you want to
    ///   load everything.
    ///
    /// # Returns
    /// A reference to the loaded source snippet.
    ///
    /// # Errors
    /// This function may error if we needed to load from a reader and the reader failed.
    ///
    /// # Panics
    /// This function panics if it didn't know the given `id`. Use [`Sources::contains_id()`] to
    /// manually check if beforehand if you need to.
    pub fn load<'s>(&'s mut self, id: u64, range: impl Into<Range>) -> Result<&'s [u8], Error> {
        let mut range: Range = range.into();

        // Step 1: Check if we have cached the requested area
        let mut sources = &mut self.sources;
        let mut toload: Range = range;
        polonius!(|sources| -> Result<&'polonius [u8], Error> {
            // Find the source and contextualize the range
            let Some(s) = sources.find(id, Source::is(id)) else { panic!("Unknown ID {id:?}") };
            range = range.contextualize(s.len);
            toload = range;

            // Then see if we have a fitting cache line
            if let Some((loaded, text)) = &s.cache {
                if loaded.overarches(toload) {
                    let pos: usize = (toload.pos - loaded.pos) as usize;
                    let len: usize = toload.len.fixed() as usize;
                    polonius_return!(Ok(&text[pos..pos + len]));
                } else {
                    // Remember that we're going to want to load more to include the old line
                    toload.extend(*loaded);
                }
            }
        });

        // Step 2: Load a new chunk
        // SAFETY: We are sure that this will succeed because we've already run this check above (but unmutably).
        let Some(s) = sources.find_mut(id, Source::is(id)) else { unreachable!() };
        s.reader.seek(SeekFrom::Start(toload.pos)).map_err(|err| Error::ReaderSeek { name: s.name.clone(), err })?;
        let mut buf: Vec<u8> = vec![0; toload.len.fixed() as usize];
        s.reader.read_exact(&mut buf).map_err(|err| Error::ReaderRead { name: s.name.clone(), err })?;
        s.cache = Some((toload, Cow::Owned(buf)));

        // Now access the cache one last time to load the data we just added
        let pos: usize = (range.pos - toload.pos) as usize;
        let len: usize = range.len.fixed() as usize;
        Ok(&s.cache.as_ref().unwrap().1[pos..pos + len])
    }

    /// Unloads a specific source text with a given ID.
    ///
    /// This frees up the memory of it if it's cached. Otherwise, it's a no-op.
    ///
    /// # Arguments
    /// - `id`: The ID of the source text to unload.
    ///
    /// # Returns
    /// True if it was present and it was unloaded, or false otherwise.
    ///
    /// # Panics
    /// Note that this function still panics if we didn't know the `id` at all (we just accept it
    /// if we know it, but it isn't cached). Use [`Sources::contains_id()`] to manually check
    /// beforehand if you need to.
    #[inline]
    #[track_caller]
    pub fn unload(&mut self, id: u64) -> bool {
        // Get the source entry
        let Some(s) = self.sources.find_mut(id, Source::is(id)) else { panic!("Unknown ID {id}") };

        // Then do the actual code
        s.cache.take().is_some()
    }

    /// Unloads a all source texts.
    ///
    /// This frees up all memory of cached source texts.
    ///
    /// # Returns
    /// True if anything was unloaded, or false otherwise.
    #[inline]
    pub fn unload_all(&mut self) -> bool {
        let mut unloaded_anything: bool = false;
        for source in self.sources.iter_mut() {
            unloaded_anything |= source.cache.take().is_some();
        }
        unloaded_anything
    }



    /// Reserves space for additional sources.
    ///
    /// # Arguments
    /// - `additional`: The minimum number of additional sources to allocate memory for. Note that
    ///   the backing allocator may choose to allocate more.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    #[track_caller]
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.sources.try_reserve(additional, Source::hash).unwrap_or_else(|err| panic!("Failed to reallocate Sources: {err}"));
        self
    }



    /// Clears all elements from the sources list.
    ///
    /// This will close all handles but keep the backing memory (except for cache memory).
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.sources.clear();
        self
    }
}

// Collection - access
impl<'r, H: BuildHasher> Sources<'r, H> {
    /// Returns the ID that was assigned to the given name.
    ///
    /// # Arguments
    /// - `name`: A name by which a reader may be known.
    ///
    /// # Returns
    /// The ID to which this name belongs, or [`None`] if we don't know the name.
    #[inline]
    pub fn get_id_of(&self, name: impl AsRef<str>) -> Option<u64> {
        // First, we hash the name
        let name: &str = name.as_ref();
        let id: u64 = self.state.hash_one(name);

        // Only return it if we know it
        if self.sources.find(id, Source::is(id)).is_some() { Some(id) } else { None }
    }

    /// Returns the name of an ID that was assigned.
    ///
    /// # Arguments
    /// - `id`: Some ID of which to return the matching name.
    ///
    /// # Returns
    /// The name to which the ID belongs, or [`None`] if we don't know the ID.
    #[inline]
    pub fn get_name_of(&self, id: u64) -> Option<&str> { self.sources.find(id, Source::is(id)).map(|s| s.name.as_str()) }

    /// Returns the reader of the inner source.
    ///
    /// Note that this function abstracts over the specific kind of reader.
    ///
    /// # Arguments
    /// - `id`: Some ID of which to get the source.
    ///
    /// # Returns
    /// Something implementing [`Read`] and [`Seek`].
    ///
    /// # Panics
    /// This function can panic if a source with the given ID was not known.
    #[inline]
    #[track_caller]
    pub fn source(&mut self, id: u64) -> &mut (impl 'r + Seek + Read) {
        self.sources.find_mut(id, Source::is(id)).map(|s| &mut s.reader).unwrap_or_else(|| panic!("Unknown ID {id}"))
    }

    /// Returns the total length of a source with the given ID.
    ///
    /// # Arguments
    /// - `id`: Some ID of which to get the source's length.
    ///
    /// # Returns
    /// The length of the source as a [`u64`].
    ///
    /// # Panics
    /// This function can panic if a source with the given ID was not known.
    #[inline]
    #[track_caller]
    pub fn source_len(&self, id: u64) -> u64 { self.sources.find(id, Source::is(id)).map(|s| s.len).unwrap_or_else(|| panic!("Unknown ID {id}")) }

    /// Returns the range of a source which is cached.
    ///
    /// This can be used to determine if a slice can be accessed cheaply.
    ///
    /// # Returns
    /// A [`Range`] describing the number- and area of cached bytes. If nothing is cached, then
    /// this returns [`None`].
    ///
    /// # Panics
    /// This function can panic if a source with the given ID was not known.
    #[inline]
    #[track_caller]
    pub fn cached_range(&self, id: u64) -> Option<Range> {
        self.sources.find(id, Source::is(id)).map(|s| s.cache.as_ref().map(|(r, _)| *r)).unwrap_or_else(|| panic!("Unknown ID {id}"))
    }

    /// Checks if the given name exists in this Sources.
    ///
    /// # Arguments
    /// - `name`: The name to check if we know it.
    ///
    /// # Returns
    /// True if we do, false if we don't.
    #[inline]
    pub fn contains_name(&self, name: impl AsRef<str>) -> bool { self.get_id_of(name).is_some() }

    /// Checks if the given ID exists in this Sources.
    ///
    /// # Arguments
    /// - `id`: The ID to check if we know it.
    ///
    /// # Returns
    /// True if we do, false if we don't.
    #[inline]
    pub fn contains_id(&self, id: u64) -> bool { self.get_name_of(id).is_some() }



    /// Returns an iterator that yields all the identifiers and names in the sources list.

    /// Returns the number of sources in this Sources.
    ///
    /// # Returns
    /// The number of added sources.
    #[inline]
    pub fn len(&self) -> usize { self.sources.len() }

    /// Returns whether there are no elements in this Sources.
    ///
    /// # Returns
    /// True if [`Sources::len()`] returns 0, or false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.sources.is_empty() }

    /// Returns the total capacity, which is the number of sources this can store in total before
    /// needing to reallocate.
    ///
    /// # Returns
    /// The number of added sources.
    #[inline]
    pub fn capacity(&self) -> usize { self.sources.capacity() }
}




/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_load() {
        let mut sources = Sources::new();
        // Don't add the string with the dedicated method to simulate loading it
        let id: u64 = sources.add("test", Cursor::new(b"Hello, world!"), 13);
        assert_eq!(sources.cached_range(id), None);
        assert_eq!(sources.load(id, ..5).unwrap(), b"Hello");
        assert_eq!(sources.cached_range(id), Some(Range::from(..5)));
        assert_eq!(sources.load(id, 7..).unwrap(), b"world!");
        assert_eq!(sources.cached_range(id), Some(Range::from(..13)));
        assert_eq!(sources.load(id, 5..7).unwrap(), b", ");
        assert_eq!(sources.cached_range(id), Some(Range::from(..13)));
        assert_eq!(sources.load(id, ..).unwrap(), b"Hello, world!");
        assert_eq!(sources.cached_range(id), Some(Range::from(..13)));

        // Do some unloading and check how things are
        assert_eq!(sources.unload(id), true);
        assert_eq!(sources.unload(id), false);
        assert_eq!(sources.unload_all(), false);
        assert_eq!(sources.cached_range(id), None);
        assert_eq!(sources.load(id, 5..7).unwrap(), b", ");
        assert_eq!(sources.cached_range(id), Some(Range::from(5..7)));
    }
}
