use crate::locale_api::parser::ParseR3Error;
use memchr::{memchr, memchr_iter};

/// Sanitizes a locale file by removing comments and normalizing line endings.
///
/// # For Students: What This Function Does
/// 1. Validates the file is valid UTF-8
/// 2. Converts Windows line endings (\r\n) to Unix (\n)
/// 3. Removes comments (lines starting with ##)
///
/// # Why We Need This
/// - Files from different OSes have different line endings
/// - Comments should not appear in the parsed data
/// - UTF-8 validation ensures we can safely work with strings
///
/// # Arguments
/// * `file` - Raw bytes of the input file
///
/// # Returns
/// * `Ok(Box<[u8]>)` - Cleaned file contents
/// * `Err(ParseR3Error::InvalidUTF8Value)` - File is not valid UTF-8
pub fn sanitize_r3_locale_file(file: &[u8]) -> Result<Box<[u8]>, ParseR3Error> {
    // Step 1: Validate UTF-8
    if std::str::from_utf8(file).is_err() {
        return Err(ParseR3Error::InvalidUTF8Value);
    }
    
    // Step 2: Normalize line endings (convert \r\n and \r to \n)
    let file_len = file.len();
    let mut temp_file = Vec::with_capacity(file_len);
    let mut last_pos = 0;
    
    // Find all carriage returns (\r) in the file
    for pos in memchr_iter(b'\r', file) {
        // Copy everything before this \r
        temp_file.extend_from_slice(&file[last_pos..pos]);
        
        // Check if it's \r\n (Windows) or just \r (old Mac)
        if file.get(pos + 1) == Some(&b'\n') {
            // Windows style: replace \r\n with \n
            temp_file.push(b'\n');
            last_pos = pos + 2;
        } else {
            // Old Mac style: replace \r with \n
            temp_file.push(b'\n');
            last_pos = pos + 1;
        }
    }
    // Copy any remaining bytes after the last \r
    temp_file.extend_from_slice(&file[last_pos..]);

    // Step 3: Remove comments (## to end of line)
    // Find all # characters
    let comment_opening_matches_initial: Vec<usize> = memchr_iter(b'#', &temp_file).collect();
    let mut comment_closing_matches = Vec::new();
    let mut comment_opening_matches: Vec<usize> =
        Vec::with_capacity(comment_opening_matches_initial.len() / 2);
    
    // Look for ## patterns (two # in a row)
    for item in &comment_opening_matches_initial {
        // Check if this # is followed by another # (and not preceded by one)
        if comment_opening_matches_initial.contains(&(item + 1))
            && (*item == 0 || !comment_opening_matches_initial.contains(&(item - 1)))
        {
            // Found ## - this starts a comment
            // Find where the comment ends (at newline or EOF)
            if let Some(close_pos) = memchr(b'\n', &temp_file[*item..]) {
                comment_opening_matches.push(*item);
                comment_closing_matches.push(Some(close_pos));
            } else {
                // Comment goes to end of file
                comment_opening_matches.push(*item);
                comment_closing_matches.push(None);
            }
        }
    }

    // Step 4: Build final file with comments removed
    let mut final_file: Vec<u8> = Vec::with_capacity(temp_file.len());
    last_pos = 0;
    
    // Clean up comment position arrays
    comment_opening_matches.dedup();
    comment_closing_matches.dedup();
    comment_opening_matches.sort();
    comment_closing_matches.sort();

    // Copy everything except comment sections
    for (&open_pos, &close_pos) in comment_opening_matches
        .iter()
        .zip(comment_closing_matches.iter())
    {
        if let Some(pos) = close_pos {
            // Comment has a closing newline
            // Copy from last position to start of comment
            final_file.extend_from_slice(&temp_file[last_pos..open_pos]);
            // Skip over the comment (including newline)
            last_pos = open_pos + pos + 1;
        } else {
            // Comment goes to end of file
            final_file.extend_from_slice(&temp_file[last_pos..open_pos]);
            last_pos = temp_file.len();
            break;
        }
    }

    // Copy any remaining content after last comment
    if last_pos < temp_file.len() {
        final_file.extend_from_slice(&temp_file[last_pos..]);
    }

    Ok(final_file.into_boxed_slice())
}
