import os

replacements = [
    ("rust_eloquent", "rullst_orm"),
    ("rust-eloquent", "rullst-orm"),
    ("EloquentModel", "RullstModel"),
    ("EloquentValue", "RullstValue"),
    ("EloquentDatabase", "RullstDatabase"),
    ("EloquentPool", "RullstPool"),
    ("EloquentCollection", "RullstCollection"),
    ("eloquent_macro", "rullst_macro"),
    ("eloquent", "rullst"),
    ("Eloquent", "Rullst")
]

directories = [
    "rullst-orm",
    "rullst-orm-macros",
    "docs",
    "scratch",
    "examples"
]

files_to_check = [
    "CHANGELOG.md",
    "ROADMAP.md",
    "README.md",
    "docs/spec.md",
    "docs/audit_report_complete.md"
]

def replace_in_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            
        original_content = content
        for old, new in replacements:
            content = content.replace(old, new)
            
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Updated {filepath}")
    except Exception as e:
        print(f"Error reading {filepath}: {e}")

for d in directories:
    for root, dirs, files in os.walk(d):
        for file in files:
            if file.endswith('.rs') or file.endswith('.md') or file.endswith('.toml'):
                filepath = os.path.join(root, file)
                replace_in_file(filepath)

for f in files_to_check:
    if os.path.exists(f):
        replace_in_file(f)

print("Done renaming!")
