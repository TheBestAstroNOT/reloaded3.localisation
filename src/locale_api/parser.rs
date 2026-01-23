use super::interop::LocaleTable;
use super::interop::TableEntry;
use super::sanitizer::sanitize_r3_locale_file;
use hashbrown::HashTable;
use memchr::{memchr, memmem};
use std::fs;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

pub fn parse_r3locale_file(path: &Path) -> Result<LocaleTable, ParseR3Error> {
    if !path.exists() {
        return Err(ParseR3Error::FileNotFound);
    }
    let bytes = fs::read(path).map_err(|_| ParseR3Error::FailedToRead)?;
    parse_r3locale_bytes(&bytes)
}

/// Parses a Reloaded 3 localisation file from raw bytes and returns a LocaleTable.
///
/// # How it works (for students):
/// 1. Sanitize input (remove comments, normalize line endings)
/// 2. Find all [[key]] patterns in the file
/// 3. Extract values between keys
/// 4. Build a hash table for O(1) lookups
/// 5. Store all values in a single contiguous buffer
///
/// # Arguments
/// * `bytes` - Raw bytes of the .r3l file
///
/// # Returns
/// * `Ok(LocaleTable)` - Successfully parsed locale table
/// * `Err(ParseR3Error)` - Parsing failed (invalid format, duplicates, etc.)
///
/// # Example
/// ```
/// use reloaded3_localisation::parse_r3locale_bytes;
/// 
/// let data = b"[[Hello]]\nWorld\n[[Bye]]\nGoodbye\n";
/// let table = parse_r3locale_bytes(data).unwrap();
/// assert_eq!(table.find_entry(b"Hello"), Some("World"));
/// ```
pub fn parse_r3locale_bytes(bytes: &[u8]) -> Result<LocaleTable, ParseR3Error> {
    // Step 1: Clean the input (remove comments, fix line endings)
    let sanitised_bytes: Box<[u8]> = match sanitize_r3_locale_file(bytes) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    // Step 2: Find all potential key positions (everywhere we see "[[")
    // Using memmem for fast binary search - much faster than string operations
    let opening_brackets_matches_initial: Vec<usize> =
        memmem::find_iter(&sanitised_bytes, b"[[").collect();
    
    // Allocate space for the valid matches (only those at line start)
    let mut opening_brackets_matches_final: Vec<usize> =
        Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut closing_brackets_matches_final: Vec<usize> =
        Vec::with_capacity(opening_brackets_matches_initial.len());
    let mut value_start: Vec<usize> = Vec::with_capacity(opening_brackets_matches_initial.len());
    
    // Step 3: Filter to only include brackets that start a line
    // Valid keys must be at position 0 OR preceded by a newline
    for item in &opening_brackets_matches_initial {
        if *item == 0 || sanitised_bytes[item - 1] == b'\n' {
            opening_brackets_matches_final.push(*item);
            // Find the closing ]] for this key
            if let Some(close_pos) = memmem::find(&sanitised_bytes[*item..], b"]]") {
                closing_brackets_matches_final.push(item + close_pos);
                
                // Find where the value starts (after the newline following ]])
                if let Some(value_open_pos) = memchr(b'\n', &sanitised_bytes[item + close_pos..]) {
                    value_start.push(item + close_pos + value_open_pos);
                } else {
                    // Key without value - error!
                    return Err(ParseR3Error::KeyValueMismatch);
                }
            } else {
                // Opening [[ without closing ]] - error!
                return Err(ParseR3Error::BracketMismatch);
            }
        }
    }

    // Step 4: Clean up and sort all our position vectors
    // dedup() removes duplicates, sort() puts them in order
    opening_brackets_matches_final.dedup();
    opening_brackets_matches_final.sort();
    closing_brackets_matches_final.dedup();
    closing_brackets_matches_final.sort();
    value_start.dedup();
    value_start.sort();

    // Step 5: Build the unified buffer and hash table
    // All values are concatenated into one buffer for memory efficiency
    let mut concatenated_value: Vec<u8> = Vec::with_capacity(sanitised_bytes.len());
    
    // Hash table maps key hashes to (offset, length) pairs
    let mut locale_hash_table: HashTable<TableEntry> = HashTable::new();
    let mut offset = 0; // Current position in the concatenated buffer
    // Step 6: Extract each key-value pair
    // Iterate through all valid key positions
    for i in 0..opening_brackets_matches_final
        .len()
        .min(closing_brackets_matches_final.len())
        .min(value_start.len())
    {
        // Extract the key text between [[ and ]]
        // Example: [[Hello]] -> "Hello"
        let key = std::str::from_utf8(
            &sanitised_bytes
                [opening_brackets_matches_final[i] + 2..closing_brackets_matches_final[i]],
        )
        .expect("Invalid UTF-8 input")
        .trim()
        .as_bytes();
        
        // Extract the value (from newline after ]] until next [[ or end of file)
        let value = std::str::from_utf8(
            &sanitised_bytes[value_start[i]
                ..*opening_brackets_matches_final
                    .get(i + 1)
                    .unwrap_or(&sanitised_bytes.len())],
        )
        .expect("Invalid UTF-8 input")
        .trim()
        .as_bytes();
        
        // Add value to our unified buffer
        concatenated_value.extend_from_slice(value);
        
        // Add entry to hash table (key_hash -> offset, length)
        if insert_into_hashtable(&mut locale_hash_table, key, offset, value.len()).is_err() {
            return Err(ParseR3Error::DuplicateKeys);
        }

        offset += value.len();
    }
    // Step 7: Finalize and return
    // Shrink buffer to exact size (save memory)
    concatenated_value.shrink_to_fit();

    Ok(LocaleTable {
        unified_box: concatenated_value.into_boxed_slice(),
        entries: locale_hash_table,
    })
}

/// Inserts a key-value entry into the hash table.
///
/// # For Students:
/// This function converts a text key into a 64-bit hash using XXH3,
/// then stores (offset, length) in the hash table. The hash allows
/// O(1) average-case lookups.
///
/// # Arguments
/// * `table` - The hash table to insert into
/// * `key` - The key as raw bytes (e.g., b"Hello")
/// * `offset` - Where this value starts in the unified buffer
/// * `length` - How many bytes the value occupies
///
/// # Returns
/// * `Ok(())` - Successfully inserted
/// * `Err(ParseR3Error::DuplicateKeys)` - Key already exists
pub fn insert_into_hashtable(
    table: &mut HashTable<TableEntry>,
    key: &[u8],
    offset: usize,
    length: usize,
) -> Result<(), ParseR3Error> {
    // Hash the key: "Hello" -> some u64 number
    let hash = xxh3_64(key);
    // Check if this hash already exists (duplicate key check)
    if table
        .find(hash, |table_entry: &TableEntry| table_entry.key == hash)
        .is_none()
    {
        // Hash not found - insert new entry
        table.insert_unique(
            hash,
            TableEntry {
                key: hash,
                offset,
                length,
            },
            move |e: &TableEntry| e.key,
        );
        Ok(())
    } else {
        // Hash already exists - duplicate key!
        Err(ParseR3Error::DuplicateKeys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_find_entry() {
        let sample =
            b"[[example_key]]hiiii\nexample_value\n##BYEE\n[[another_key]]\nanother_value\n";
        let table = parse_r3locale_bytes(sample).expect("Parse failed");

        let val = table.find_entry(b"example_key");
        assert_eq!(val, Some("example_value"));

        let val2 = table.find_entry(b"another_key");
        assert_eq!(val2, Some("another_value"));

        let missing = table.find_entry(b"missing");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_invalid_utf8() {
        let sample = b"[[bad_key]]\n\xFF\xFE\xFD\n";
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::InvalidUTF8Value)));
    }

    #[test]
    fn test_key_value_mismatch() {
        let sample = b"[[only_key]]"; // no value
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::KeyValueMismatch)));
    }

    #[test]
    fn test_bracket_mismatch() {
        let sample = b"[[no_close\nvalue here\n";
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::BracketMismatch)));
    }

    #[test]
    fn test_duplicate_keys() {
        let sample = b"[[duplicate_key]]\nfirst_value\n[[duplicate_key]]\nsecond_value";
        let result = parse_r3locale_bytes(sample);
        assert!(matches!(result, Err(ParseR3Error::DuplicateKeys)));
    }
}

#[repr(C)]
pub struct MergeResult {
    pub table: *mut LocaleTable,
    pub merge_state: MergeTableError,
}

pub fn get_locale_table_rust(path: &Path) -> Result<LocaleTable, ParseR3Error> {
    parse_r3locale_file(path)
}

pub fn merge_locale_table_rust(tables: &[&LocaleTable]) -> MergeResult {
    let initial_hasher = |entry: &(TableEntry, &Box<[u8]>)| entry.0.key;
    let final_hasher = |entry: &TableEntry| entry.key;
    let mut initial_table: HashTable<(TableEntry, &Box<[u8]>)> = HashTable::new();

    for table in tables {
        for entry in table.entries.iter() {
            if initial_table
                .find(entry.key, |table_entry: &(TableEntry, &Box<[u8]>)| {
                    table_entry.0.key == entry.key
                })
                .is_none()
            {
                initial_table.insert_unique(
                    entry.key,
                    (*entry, &table.unified_box),
                    initial_hasher,
                );
            }
        }
    }

    let mut final_table: HashTable<TableEntry> = HashTable::new();
    let mut final_buffer: Vec<u8> = Vec::new();
    for entry in initial_table.iter() {
        final_table.insert_unique(
            entry.0.key,
            TableEntry {
                key: entry.0.key,
                length: entry.0.length,
                offset: final_buffer.len(),
            },
            final_hasher,
        );
        final_buffer.extend_from_slice(&entry.1[entry.0.offset..entry.0.offset + entry.0.length]);
    }

    let final_boxed_buffer = final_buffer.into_boxed_slice();
    MergeResult {
        table: Box::into_raw(Box::new(LocaleTable {
            unified_box: final_boxed_buffer,
            entries: final_table,
        })),
        merge_state: MergeTableError::Normal,
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum ParseR3Error {
    Normal,
    FileNotFound,
    FailedToRead,
    KeyValueMismatch,
    BracketMismatch,
    InvalidUTF8Value,
    InvalidUTF8Path,
    NullPathProvided,
    DuplicateKeys,
}

#[derive(Debug)]
#[repr(C)]
pub enum MergeTableError {
    Normal,
    NullTablePointer,
    FileNotFound,
    FailedToRead,
    KeyValueMismatch,
    BracketMismatch,
    InvalidUTF8Value,
    InvalidUTF8Path,
    NullPathProvided,
    DuplicateKeys,
}

impl From<ParseR3Error> for MergeTableError {
    fn from(err: ParseR3Error) -> Self {
        match err {
            ParseR3Error::Normal => MergeTableError::Normal,
            ParseR3Error::FileNotFound => MergeTableError::FileNotFound,
            ParseR3Error::FailedToRead => MergeTableError::FailedToRead,
            ParseR3Error::KeyValueMismatch => MergeTableError::KeyValueMismatch,
            ParseR3Error::BracketMismatch => MergeTableError::BracketMismatch,
            ParseR3Error::InvalidUTF8Value => MergeTableError::InvalidUTF8Value,
            ParseR3Error::InvalidUTF8Path => MergeTableError::InvalidUTF8Path,
            ParseR3Error::NullPathProvided => MergeTableError::NullPathProvided,
            ParseR3Error::DuplicateKeys => MergeTableError::DuplicateKeys,
        }
    }
}
