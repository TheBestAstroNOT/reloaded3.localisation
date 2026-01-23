# Examples Directory

This directory contains example `.r3l` locale files for learning and testing.

## Files

### `student_examples.r3l`
A comprehensive example showing:
- Basic key-value pairs
- Multi-line values
- Comments (##)
- Descriptive key naming conventions

**Perfect for:** First-time users, students learning the format

### `spanish.r3l`
Spanish translations of the same keys from `student_examples.r3l`

**Perfect for:** Testing multi-language support, merge operations

### `french.r3l`
French translations of the same keys from `student_examples.r3l`

**Perfect for:** Testing multi-language support, merge operations

## How to Use These Examples

### 1. Parse an example file

```rust
use reloaded3_localisation::parse_r3locale_bytes;
use std::fs;

fn main() {
    let bytes = fs::read("examples/student_examples.r3l")
        .expect("Failed to read file");
    
    let table = parse_r3locale_bytes(&bytes)
        .expect("Failed to parse");
    
    println!("Parsed {} entries", table.entries.len());
    
    if let Some(msg) = table.find_entry(b"Hello") {
        println!("Hello message: {}", msg);
    }
}
```

### 2. Test different languages

```rust
use reloaded3_localisation::parse_r3locale_bytes;
use std::fs;

fn load_language(lang: &str) -> String {
    let path = format!("examples/{}.r3l", lang);
    let bytes = fs::read(&path).unwrap();
    let table = parse_r3locale_bytes(&bytes).unwrap();
    
    table.find_entry(b"Hello")
        .unwrap_or("Translation not found")
        .to_string()
}

fn main() {
    println!("English: {}", load_language("student_examples"));
    println!("Spanish: {}", load_language("spanish"));
    println!("French: {}", load_language("french"));
}
```

### 3. Merge language files

```rust
use reloaded3_localisation::{parse_r3locale_bytes, merge_locale_table_rust};
use std::fs;

fn main() {
    // Load base language (English)
    let base_bytes = fs::read("examples/student_examples.r3l").unwrap();
    let base_table = parse_r3locale_bytes(&base_bytes).unwrap();
    
    // Load override language (Spanish)
    let spanish_bytes = fs::read("examples/spanish.r3l").unwrap();
    let spanish_table = parse_r3locale_bytes(&spanish_bytes).unwrap();
    
    // Merge (base takes priority)
    let merged = merge_locale_table_rust(&[&base_table, &spanish_table]);
    
    println!("Merged table created successfully!");
}
```

## Practice Exercises

### Easy
1. Parse `student_examples.r3l` and print all entries
2. Count how many keys are in each language file
3. Write code to check if a key exists in all three files

### Medium
4. Create your own `.r3l` file with custom translations
5. Write a function that tries each language file until it finds a key
6. Merge all three files and verify the result

### Advanced
7. Create a CLI tool that takes a key and language code as arguments
8. Build a language switcher that can dynamically load files
9. Write tests that validate all three files have the same keys

## Creating Your Own Example Files

Follow this template:

```
##Description of this file

[[FirstKey]]
First value

[[SecondKey]]
Second value can
span multiple
lines

##You can add comments anywhere

[[ThirdKey]]
Third value
```

**Rules:**
- Keys must be in double brackets: `[[Key]]`
- Keys must start on a new line
- Values start after the `]]` and continue until the next `[[` or EOF
- Comments start with `##` and are ignored
- File must be valid UTF-8

## Next Steps

After working with these examples:
1. Read the [Beginner Tutorial](../docs/BEGINNER_TUTORIAL.md)
2. Check out the [Student Guide](../docs/STUDENT_GUIDE.md)
3. Look at the tests in `src/locale_api/parser.rs`
4. Try modifying these files and see what happens!

Happy learning! 🦀
