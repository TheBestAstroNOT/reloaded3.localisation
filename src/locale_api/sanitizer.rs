use crate::locale_api::parser::ParseR3Error;
use memchr::{memchr, memmem};

pub fn sanitize_r3_locale_file(file: &mut [u8]) -> Result<(), ParseR3Error> {
    if std::str::from_utf8(file).is_err() {
        return Err(ParseR3Error::InvalidUTF8Value);
    }

    let comment_opening_matches: Vec<usize> = memmem::find_iter(&file, "##").collect();
    for item in &comment_opening_matches {
        if let Some(close_pos) = memchr(b'\n', &file[*item..]) {
            file[*item..*item + close_pos].fill(b' ');
        }
    }

    Ok(())
}