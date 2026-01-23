# Beginner Tutorial: Using the Reloaded3 Localisation Library

Welcome! This tutorial will walk you through using this library step by step.

## Prerequisites

- Rust installed ([rustup.rs](https://rustup.rs/))
- Basic Rust knowledge (variables, functions, Result type)
- A text editor

## Tutorial 1: Your First Locale File

### Step 1: Create a simple locale file

Create a file named `greetings.r3l`:

```
[[Hello]]
Hello, World!

[[Goodbye]]
See you later!

[[Thanks]]
Thank you very much!
```

### Step 2: Parse the file in Rust

Create a new Rust project or add this to your `main.rs`:

```rust
use reloaded3_localisation::parse_r3locale_bytes;
use std::fs;

fn main() {
    // Read the file
    let bytes = fs::read("greetings.r3l")
        .expect("Failed to read file");
    
    // Parse it
    let table = parse_r3locale_bytes(&bytes)
        .expect("Failed to parse");
    
    // Look up a translation
    if let Some(greeting) = table.find_entry(b"Hello") {
        println!("Translation: {}", greeting);
    }
}
```

### Step 3: Run it

```bash
cargo run
```

You should see: `Translation: Hello, World!`

---

## Tutorial 2: Understanding the Format

### Valid Format Examples

✅ **Basic key-value:**
```
[[Key]]
Value
```

✅ **Multiple lines in value:**
```
[[Message]]
This is line 1
This is line 2
This is line 3
```

✅ **With comments:**
```
##This is a comment - it will be ignored
[[Key]]
Value
```

### Invalid Format Examples

❌ **Missing value:**
```
[[KeyWithoutValue]]
```
Error: `KeyValueMismatch`

❌ **Missing closing bracket:**
```
[[Incomplete
Value
```
Error: `BracketMismatch`

❌ **Duplicate keys:**
```
[[Same]]
First value
[[Same]]
Second value
```
Error: `DuplicateKeys`

---

## Tutorial 3: Error Handling

```rust
use reloaded3_localisation::{parse_r3locale_bytes, ParseR3Error};

fn main() {
    let bytes = b"[[Hello]]\nWorld";
    
    match parse_r3locale_bytes(bytes) {
        Ok(table) => {
            println!("Parsed successfully!");
            println!("Entries: {:?}", table.entries.len());
        }
        Err(ParseR3Error::BracketMismatch) => {
            println!("Error: Brackets don't match!");
        }
        Err(ParseR3Error::DuplicateKeys) => {
            println!("Error: Duplicate keys found!");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
```

---

## Tutorial 4: Working with Multiple Languages

### Step 1: Create language files

**english.r3l:**
```
[[Greeting]]
Hello!

[[Farewell]]
Goodbye!
```

**spanish.r3l:**
```
[[Greeting]]
¡Hola!

[[Farewell]]
¡Adiós!
```

**french.r3l:**
```
[[Greeting]]
Bonjour!

[[Farewell]]
Au revoir!
```

### Step 2: Load the appropriate language

```rust
use reloaded3_localisation::{parse_r3locale_bytes, LocaleTable, ParseR3Error};
use std::fs;

fn load_language(lang: &str) -> Result<LocaleTable, ParseR3Error> {
    let filename = format!("{}.r3l", lang);
    let bytes = fs::read(&filename).map_err(|_| ParseR3Error::FailedToRead)?;
    parse_r3locale_bytes(&bytes)
}

fn main() {
    let lang = "spanish"; // Change this to switch languages
    
    match load_language(lang) {
        Ok(table) => {
            if let Some(greeting) = table.find_entry(b"Greeting") {
                println!("{}", greeting);
            }
        }
        Err(e) => println!("Failed to load language: {:?}", e),
    }
}
```

---

## Tutorial 5: Merging Multiple Locale Tables

Sometimes you want to combine multiple locale files (e.g., base language + customizations).

```rust
use reloaded3_localisation::merge_locale_table_rust;

fn main() {
    // Parse two separate files
    let base = parse_r3locale_bytes(b"[[A]]\nBase value A\n[[B]]\nBase value B").unwrap();
    let custom = parse_r3locale_bytes(b"[[B]]\nCustom value B\n[[C]]\nCustom value C").unwrap();
    
    // Merge them (first table wins for duplicates)
    let merged = merge_locale_table_rust(&[&base, &custom]);
    
    // merged now contains:
    // [[A]] -> "Base value A"
    // [[B]] -> "Base value B"  (base wins)
    // [[C]] -> "Custom value C"
}
```

---

## Tutorial 6: Best Practices

### 1. **Always validate input files**

```rust
fn load_locale_safe(path: &str) -> Result<LocaleTable, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("File error: {}", e))?;
    
    parse_r3locale_bytes(&bytes)
        .map_err(|e| format!("Parse error: {:?}", e))
}
```

### 2. **Use descriptive key names**

✅ Good:
```
[[UI_Button_Save]]
[[Error_FileNotFound]]
[[Menu_Settings_Audio_Volume]]
```

❌ Bad:
```
[[K1]]
[[Msg]]
[[X]]
```

### 3. **Group related translations**

```
[[Menu_File_New]]
New File

[[Menu_File_Open]]
Open File

[[Menu_File_Save]]
Save File

[[Menu_Edit_Copy]]
Copy

[[Menu_Edit_Paste]]
Paste
```

### 4. **Document your key structure**

Create a `KEYS.md` file:
```markdown
# Translation Keys

## UI Elements
- `UI_Button_*` - Button labels
- `UI_Menu_*` - Menu items
- `UI_Dialog_*` - Dialog text

## Error Messages
- `Error_*` - Error messages

## Game Content
- `Character_*` - Character names
- `Item_*` - Item descriptions
```

---

## Common Mistakes to Avoid

### 1. Forgetting the newline after ]]

❌ Wrong:
```
[[Key]]Value
```

✅ Correct:
```
[[Key]]
Value
```

### 2. Using single brackets

❌ Wrong:
```
[Key]
Value
```

✅ Correct:
```
[[Key]]
Value
```

### 3. Not trimming whitespace

The library automatically trims whitespace, so these are equivalent:
```
[[Key]]
   Value   
```
```
[[Key]]
Value
```

But be consistent for readability!

---

## Next Steps

Once you're comfortable with the basics:

1. Read the [Student Guide](STUDENT_GUIDE.md) for deeper understanding
2. Look at the source code in `src/locale_api/parser.rs`
3. Try the suggested projects in the Student Guide
4. Explore the C FFI bindings (advanced)

---

## Quick Reference

### File Format
```
[[Key]]
Value

##Comment (ignored)

[[AnotherKey]]
Multi-line
value
content
```

### Basic API

```rust
// Parse
let table = parse_r3locale_bytes(&bytes)?;

// Lookup
let value = table.find_entry(b"Key");

// Merge
let merged = merge_locale_table_rust(&[&table1, &table2]);
```

### Error Types

- `FileNotFound` - File doesn't exist
- `FailedToRead` - Can't read file
- `BracketMismatch` - `[[` without `]]`
- `KeyValueMismatch` - Key without value
- `InvalidUTF8Value` - Not valid UTF-8
- `DuplicateKeys` - Same key appears twice

---

## Need Help?

- Check the [full documentation](https://docs.rs/reloaded3_localisation)
- Read the [Student Guide](STUDENT_GUIDE.md)
- Look at the tests in `src/locale_api/parser.rs`
- Open an issue on GitHub

Happy coding! 🦀
