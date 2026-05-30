import os
import re

def refactor():
    src_dir = "rullst-orm-macros/src"
    with open(f"{src_dir}/lib.rs", "r", encoding="utf-8") as f:
        content = f.read()
    
    # We will backup lib.rs
    with open(f"{src_dir}/lib.rs.bak", "w", encoding="utf-8") as f:
        f.write(content)
        
    print("Backup created.")

if __name__ == "__main__":
    refactor()
